/// Main entry point for the Habit Tracker MCP server
/// 
/// This file sets up logging, parses command line arguments, and starts the MCP server.
/// The server listens for JSON-RPC requests over stdin/stdout following the MCP protocol.

use clap::Parser;
use std::path::PathBuf;
use tracing::info;

use habit_tracker_mcp::HabitTrackerServer;

/// Command line arguments for the Habit Tracker MCP server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the SQLite database file
    /// If not provided, uses a default location in the user's home directory
    #[arg(long)]
    database: Option<PathBuf>,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// Enable verbose output (implies debug)
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Set up logging based on command line flags
    let log_level = if args.verbose {
        "debug"
    } else if args.debug {
        "info"
    } else {
        "warn"
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(format!("habit_tracker_mcp={}", log_level))
        .with_writer(std::io::stderr) // Send logs to stderr, not stdout
        .init();
    
    info!("Starting Habit Tracker MCP server");
    
    // Determine database path
    let db_path = match args.database {
        Some(path) => path,
        None => {
            // Use a default path in the user's home directory
            let mut path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."));
            path.push(".habit_tracker");
            std::fs::create_dir_all(&path)?;
            path.push("habits.db");
            path
        }
    };
    
    info!("Using database at: {}", db_path.display());
    
    // Create and start the habit tracker server
    let server = HabitTrackerServer::new(db_path).await?;
    
    // Run the MCP server - this will handle JSON-RPC communication over stdin/stdout
    server.run().await?;
    
    info!("Habit Tracker MCP server shutdown complete");
    Ok(())
}