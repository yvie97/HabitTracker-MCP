/// MCP server implementation that handles JSON-RPC communication
/// 
/// This module implements the actual MCP server that:
/// 1. Reads JSON-RPC requests from stdin
/// 2. Processes tool calls using our habit tracker
/// 3. Sends JSON-RPC responses to stdout

use std::collections::HashMap;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

use crate::mcp::protocol::*;
use crate::tools;
use crate::{HabitTrackerServer, ServerError, InsightsParams};

/// MCP server that handles communication with Claude
pub struct McpServer {
    /// The underlying habit tracker server
    habit_tracker: HabitTrackerServer,
    /// Whether the server has been initialized
    initialized: bool,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(habit_tracker: HabitTrackerServer) -> Self {
        Self {
            habit_tracker,
            initialized: false,
        }
    }
    
    /// Run the MCP server, handling JSON-RPC over stdin/stdout
    pub async fn run(&mut self) -> Result<(), ServerError> {
        info!("Starting MCP server, waiting for JSON-RPC requests...");
        
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = tokio::io::stdout();
        
        let mut line = String::new();
        
        loop {
            line.clear();
            
            // Read one line from stdin
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("MCP server shutting down (stdin closed)");
                    break;
                }
                Ok(_) => {
                    // Process the line
                    if let Some(response) = self.process_line(&line).await {
                        let response_str = serde_json::to_string(&response)?;
                        
                        // Write response + newline
                        stdout.write_all(response_str.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                        
                        debug!("Sent response: {}", response_str);
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single line of JSON-RPC input
    async fn process_line(&mut self, line: &str) -> Option<JsonRpcResponse> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        
        debug!("Processing request: {}", line);
        
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                return Some(JsonRpcResponse::error(
                    json!(null),
                    error_codes::PARSE_ERROR,
                    format!("Invalid JSON: {}", e),
                    None
                ));
            }
        };
        
        Some(self.handle_request(request).await)
    }
    
    /// Handle a JSON-RPC request
    async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "initialized" => {
                self.initialized = true;
                JsonRpcResponse::success(request.id, json!(null))
            }
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,
            _ => {
                JsonRpcResponse::error(
                    request.id,
                    error_codes::METHOD_NOT_FOUND,
                    format!("Method '{}' not found", request.method),
                    None
                )
            }
        }
    }
    
    /// Handle MCP initialization request
    async fn handle_initialize(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        info!("MCP client connected");
        
        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
            },
            server_info: ServerInfo {
                name: "Habit Tracker MCP".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }
    
    /// Handle tools/list request
    async fn handle_tools_list(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tools = vec![
            ToolDefinition {
                name: "habit_create".to_string(),
                description: "Create a new habit to track".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Name of the habit"},
                        "category": {"type": "string", "description": "Category (health, productivity, etc.)"},
                        "frequency": {"type": "string", "description": "How often (daily, weekdays, etc.)"}
                    },
                    "required": ["name", "category", "frequency"]
                }),
            },
            ToolDefinition {
                name: "habit_log".to_string(),
                description: "Log completion of a habit for today or a specific date".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "habit_id": {"type": "string", "description": "ID of the habit to log"},
                        "completed_at": {"type": "string", "description": "Date completed (YYYY-MM-DD, optional - defaults to today)"},
                        "value": {"type": "number", "description": "Amount completed (optional, e.g., 30 minutes)"},
                        "intensity": {"type": "number", "description": "Intensity rating 1-10 (optional)"},
                        "notes": {"type": "string", "description": "Optional notes about this completion"}
                    },
                    "required": ["habit_id"]
                }),
            },
            ToolDefinition {
                name: "habit_list".to_string(),
                description: "List all habits with detailed information including streaks, completion rates, and sorting options".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "category": {"type": "string", "description": "Filter by category (health, productivity, etc.) - optional"},
                        "active_only": {"type": "boolean", "description": "Show only active habits (default: true) - optional"},
                        "sort_by": {"type": "string", "description": "Sort by: 'name', 'streak', 'completion_rate', 'total_completions' (default: name) - optional"}
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "habit_status".to_string(),
                description: "Check habit status, streaks and progress".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "habit_id": {"type": "string", "description": "ID of specific habit (optional - shows all if omitted)"},
                        "include_recent": {"type": "boolean", "description": "Include recent completion history (optional)"}
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "habit_insights".to_string(),
                description: "Get AI-powered insights and recommendations for your habits".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "habit_id": {"type": "string", "description": "ID of specific habit (optional - analyzes all habits if omitted)"},
                        "time_period": {"type": "string", "description": "Analysis period: 'week', 'month', 'quarter', 'year' (optional, defaults to 'month')"},
                        "insight_type": {"type": "string", "description": "Type of insights: 'performance', 'recommendations', 'patterns', 'all' (optional, defaults to 'all')"}
                    },
                    "required": []
                }),
            },
        ];
        
        JsonRpcResponse::success(request.id, json!({"tools": tools}))
    }
    
    /// Handle tools/call request
    async fn handle_tools_call(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tool_params: ToolCallParams = match request.params {
            Some(params) => match serde_json::from_value(params) {
                Ok(p) => p,
                Err(e) => {
                    return JsonRpcResponse::error(
                        request.id,
                        error_codes::INVALID_PARAMS,
                        format!("Invalid parameters: {}", e),
                        None
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(
                    request.id,
                    error_codes::INVALID_PARAMS,
                    "Missing parameters".to_string(),
                    None
                );
            }
        };
        
        let result = match tool_params.name.as_str() {
            "habit_create" => self.call_habit_create(tool_params.arguments).await,
            "habit_log" => self.call_habit_log(tool_params.arguments).await,
            "habit_list" => self.call_habit_list(tool_params.arguments).await,
            "habit_status" => self.call_habit_status(tool_params.arguments).await,
            "habit_insights" => self.call_habit_insights(tool_params.arguments).await,
            _ => ToolCallResult::error(format!("Unknown tool: {}", tool_params.name)),
        };
        
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }
    
    /// Call the habit_create tool
    async fn call_habit_create(&self, args: HashMap<String, Value>) -> ToolCallResult {
        let create_params = tools::CreateHabitParams {
            name: args.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: None,
            category: args.get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("personal")
                .to_string(),
            frequency: args.get("frequency")
                .and_then(|v| v.as_str())
                .unwrap_or("daily")
                .to_string(),
            target_value: None,
            unit: None,
        };
        
        match tools::create_habit(self.habit_tracker.storage(), create_params) {
            Ok(response) => {
                let message = if let Some(habit_id) = &response.habit_id {
                    format!("{}\nHabit ID: {}", response.message, habit_id)
                } else {
                    response.message
                };
                ToolCallResult::success(message)
            },
            Err(e) => ToolCallResult::error(e.to_string()),
        }
    }
    
    /// Call the habit_log tool
    async fn call_habit_log(&self, args: HashMap<String, Value>) -> ToolCallResult {
        let log_params = tools::LogHabitParams {
            habit_id: args.get("habit_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            completed_at: args.get("completed_at")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            value: args.get("value")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32),
            intensity: args.get("intensity")
                .and_then(|v| v.as_u64())
                .map(|n| n as u8),
            notes: args.get("notes")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };
        
        match tools::log_habit(self.habit_tracker.storage(), log_params) {
            Ok(response) => ToolCallResult::success(response.message),
            Err(e) => ToolCallResult::error(e.to_string()),
        }
    }
    
    /// Call the habit_status tool
    async fn call_habit_status(&self, args: HashMap<String, Value>) -> ToolCallResult {
        let status_params = tools::StatusParams {
            habit_id: args.get("habit_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            include_recent: Some(args.get("include_recent")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)),
        };
        
        match tools::get_habit_status(self.habit_tracker.storage(), status_params) {
            Ok(response) => ToolCallResult::success(response.message),
            Err(e) => ToolCallResult::error(e.to_string()),
        }
    }
    
    /// Call the habit_insights tool
    async fn call_habit_insights(&self, args: HashMap<String, Value>) -> ToolCallResult {
        let insights_params = InsightsParams {
            habit_id: args.get("habit_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            time_period: args.get("time_period")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            insight_type: args.get("insight_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };
        
        match tools::get_habit_insights(self.habit_tracker.storage(), insights_params) {
            Ok(response) => ToolCallResult::success(response.message),
            Err(e) => ToolCallResult::error(e.to_string()),
        }
    }
    
    /// Call the habit_list tool
    async fn call_habit_list(&self, args: HashMap<String, Value>) -> ToolCallResult {
        let list_params = tools::ListHabitsParams {
            category: args.get("category")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            active_only: args.get("active_only")
                .and_then(|v| v.as_bool())
                .or(Some(true)), // Default to active only
            sort_by: args.get("sort_by")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        match tools::list_habits(self.habit_tracker.storage(), list_params) {
            Ok(response) => {
                if response.habits.is_empty() {
                    ToolCallResult::success("No habits found. Create your first habit to get started!".to_string())
                } else {
                    let summary = format!("📋 **Habit Summary** ({} habits)\n\n", response.summary.total_habits);

                    let detailed_list = response.habits.iter()
                        .map(|h| {
                            format!("🎯 **{}** ({})\n   📅 Frequency: {} | 🔥 Streak: {} days | 📊 Rate: {:.1}% | ✅ Total: {}{}",
                                h.name,
                                h.category,
                                h.frequency,
                                h.current_streak,
                                h.completion_rate * 100.0,
                                h.total_completions,
                                if h.is_active { "" } else { " ⏸️ (paused)" }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    let overall_stats = format!("\n\n📊 **Overall Stats**\n- Active habits: {}\n- Average completion rate: {:.1}%",
                        response.summary.active_habits,
                        response.summary.avg_completion_rate * 100.0
                    );

                    ToolCallResult::success(format!("{}{}{}", summary, detailed_list, overall_stats))
                }
            },
            Err(e) => ToolCallResult::error(e.to_string()),
        }
    }
}