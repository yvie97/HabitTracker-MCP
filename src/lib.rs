/// Public library interface for the Habit Tracker MCP server
/// 
/// This module exports the main server implementation and public types
/// that can be used by other applications or tests.

use std::path::PathBuf;
use thiserror::Error;

// Internal modules
mod domain;
mod storage; 
mod analytics;
mod tools;
mod mcp;

// Re-export public modules and types
pub use domain::*;
pub use storage::{SqliteStorage, StorageError, HabitStorage};
pub use analytics::{AnalyticsEngine, Insight, InsightsParams, InsightsResponse};

/// Errors that can occur during server operation
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Database error: {0}")]
    Database(#[from] storage::StorageError),
    
    #[error("Domain validation error: {0}")]
    Domain(#[from] domain::DomainError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Main habit tracker server that implements the MCP protocol
/// 
/// This server manages habit data through a SQLite database and provides
/// tools for creating habits, logging completions, and generating insights.
pub struct HabitTrackerServer {
    storage: SqliteStorage,
    analytics: AnalyticsEngine,
}

impl HabitTrackerServer {
    /// Create a new habit tracker server with the specified database path
    /// 
    /// This will initialize the SQLite database with the required schema
    /// if it doesn't already exist.
    pub async fn new(db_path: PathBuf) -> Result<Self, ServerError> {
        tracing::info!("Initializing Habit Tracker server with database: {:?}", db_path);
        
        // Initialize storage layer
        let storage = SqliteStorage::new(db_path)?;
        
        // Initialize analytics engine with the storage reference
        let analytics = AnalyticsEngine::new();
        
        Ok(Self {
            storage,
            analytics,
        })
    }
    
    /// Run the MCP server, handling JSON-RPC requests over stdin/stdout
    /// 
    /// This method will block until the server is shut down or an error occurs.
    pub async fn run(self) -> Result<(), ServerError> {
        tracing::info!("Starting MCP server...");
        
        // Test database connectivity
        let habits = self.storage.list_habits(None, true)?;
        tracing::info!("Server started successfully, found {} existing habits", habits.len());
        
        // Create and run the MCP server
        let mut mcp_server = mcp::McpServer::new(self);
        mcp_server.run().await?;
        
        Ok(())
    }
    
    /// Get a reference to the storage layer (useful for testing)
    pub fn storage(&self) -> &SqliteStorage {
        &self.storage
    }
    
    /// Get a reference to the analytics engine (useful for testing)
    pub fn analytics(&self) -> &AnalyticsEngine {
        &self.analytics
    }
}