/// MCP (Model Context Protocol) message structures and JSON-RPC handling
/// 
/// This module defines the JSON-RPC message format that Claude and other
/// MCP clients use to communicate with our habit tracker server.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// MCP protocol version we support
pub const MCP_VERSION: &str = "2024-11-05";

/// JSON-RPC 2.0 request message
///
/// This is the standard format for JSON-RPC requests that MCP uses.
/// When Claude wants to call a tool, it sends a message in this format.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0")
    #[allow(dead_code)]
    pub jsonrpc: String,
    /// Unique identifier for this request
    pub id: Value,
    /// The method/tool name to call (e.g., "tools/call")
    pub method: String,
    /// Parameters for the method call
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response message
/// 
/// This is what we send back to Claude after processing a request.
/// It contains either a successful result or an error.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request ID that we're responding to
    pub id: Value,
    /// Successful result (if no error occurred)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error information (if something went wrong)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error information
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    /// Error code (standard JSON-RPC codes)
    pub code: i32,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP tool call parameters
/// 
/// When Claude calls a tool, it sends parameters in this format.
#[derive(Debug, Deserialize)]
pub struct ToolCallParams {
    /// Name of the tool to call (e.g., "habit_create")
    pub name: String,
    /// Arguments to pass to the tool
    #[serde(default)]
    pub arguments: HashMap<String, Value>,
}

/// MCP tool call result
/// 
/// This is what we return after successfully executing a tool.
#[derive(Debug, Serialize)]
pub struct ToolCallResult {
    /// Tool execution results
    pub content: Vec<ToolContent>,
    /// Whether this is an error result
    #[serde(default)]
    pub is_error: bool,
}

/// Content returned by a tool
#[derive(Debug, Serialize)]
pub struct ToolContent {
    /// Type of content (usually "text")
    #[serde(rename = "type")]
    pub content_type: String,
    /// The actual content/result
    pub text: String,
}

/// MCP tool definition
/// 
/// This describes what tools our server provides to Claude.
#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    /// Tool name (e.g., "habit_create")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON schema for the tool's input parameters
    pub input_schema: Value,
}

/// MCP server capabilities
/// 
/// This tells Claude what features our server supports.
#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    /// Tools that this server provides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

/// Tools capability information
#[derive(Debug, Serialize)]
pub struct ToolsCapability {
    /// Whether we support listing available tools
    #[serde(default)]
    pub list_changed: bool,
}

/// MCP initialization request
#[derive(Debug, Deserialize)]
pub struct InitializeParams {
    /// MCP protocol version the client supports
    #[allow(dead_code)]
    pub protocol_version: String,
    /// Capabilities the client supports
    #[allow(dead_code)]
    pub capabilities: Value,
    /// Client information
    #[allow(dead_code)]
    pub client_info: ClientInfo,
}

/// Information about the MCP client (Claude)
#[derive(Debug, Deserialize)]
pub struct ClientInfo {
    /// Client name (e.g., "Claude")
    #[allow(dead_code)]
    pub name: String,
    /// Client version
    #[allow(dead_code)]
    pub version: String,
}

/// MCP initialization response
#[derive(Debug, Serialize)]
pub struct InitializeResult {
    /// MCP protocol version we support
    pub protocol_version: String,
    /// Our server capabilities
    pub capabilities: ServerCapabilities,
    /// Information about our server
    pub server_info: ServerInfo,
}

/// Information about our habit tracker server
#[derive(Debug, Serialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

// JSON-RPC error codes (standard codes)
#[allow(dead_code)] // These constants are defined for completeness and future use
pub mod error_codes {
    /// Parse error - Invalid JSON was received by the server
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid Request - The JSON sent is not a valid Request object
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found - The requested method doesn't exist
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid parameters - Method exists but parameters are wrong
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error - Internal JSON-RPC error
    pub const INTERNAL_ERROR: i32 = -32603;

    // Application-specific error codes (as per JSON-RPC 2.0 spec, these should be in -32000 to -32099 range)
    /// Habit not found - The specified habit ID doesn't exist
    pub const HABIT_NOT_FOUND: i32 = -32001;
    /// Duplicate entry - An entry already exists for this habit on this date
    pub const DUPLICATE_ENTRY: i32 = -32002;
    /// Validation error - Input validation failed
    pub const VALIDATION_ERROR: i32 = -32003;
    /// Storage error - Database or storage operation failed
    pub const STORAGE_ERROR: i32 = -32004;
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }
    
    /// Create an error response
    pub fn error(id: Value, code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data,
            }),
        }
    }
}

impl ToolCallResult {
    /// Create a successful tool result with text content
    pub fn success(text: String) -> Self {
        Self {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text,
            }],
            is_error: false,
        }
    }

    /// Create an error tool result
    pub fn error(error_message: String) -> Self {
        Self {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: format!("Error: {}", error_message),
            }],
            is_error: true,
        }
    }
}

/// Helper function to map storage errors to appropriate JSON-RPC error codes
#[allow(dead_code)] // This function is defined for future use in more detailed error reporting
pub fn storage_error_to_json_rpc_code(error: &crate::storage::StorageError) -> i32 {
    use crate::storage::StorageError;

    match error {
        StorageError::HabitNotFound { .. } => error_codes::HABIT_NOT_FOUND,
        StorageError::EntryNotFound { .. } => error_codes::HABIT_NOT_FOUND, // Reuse same code
        StorageError::DuplicateEntry { .. } => error_codes::DUPLICATE_ENTRY,
        StorageError::Query(_) => error_codes::STORAGE_ERROR,
        StorageError::Connection(_) => error_codes::STORAGE_ERROR,
        StorageError::Serialization(_) => error_codes::INTERNAL_ERROR,
        StorageError::Migration(_) => error_codes::STORAGE_ERROR,
    }
}