/// MCP tools for habit management
/// 
/// This module contains all the MCP tools that external clients (like Claude)
/// can call to interact with the habit tracker.

// Tool implementations will go in separate files
pub mod create;
pub mod log;
pub mod status;
pub mod list;
pub mod insights;
pub mod update;

// Re-export tool functions for easy access
pub use create::*;
pub use log::*;
pub use status::*;
pub use list::*;
pub use insights::*;
pub use update::*;