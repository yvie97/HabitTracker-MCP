/// MCP protocol implementation
/// 
/// This module handles the Model Context Protocol communication,
/// including JSON-RPC parsing and tool routing.

pub mod protocol;
pub mod server;

// Re-export main types
pub use server::McpServer;