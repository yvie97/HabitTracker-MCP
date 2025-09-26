/// Main entry point for the Habit Tracker MCP server
/// 
/// This file sets up logging, parses command line arguments, and starts the MCP server.
/// The server listens for JSON-RPC requests over stdin/stdout following the MCP protocol.

use clap::Parser;
use std::path::PathBuf;
use tracing::info;

use habit_tracker_mcp::HabitTrackerServer;

/// Get the default database path with robust fallback strategy
fn get_default_database_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Try various locations in order of preference
    let potential_paths = [
        // 1. User's home directory (preferred)
        dirs::home_dir().map(|mut p| {
            p.push(".habit_tracker");
            p
        }),
        // 2. User's data directory (platform-specific)
        dirs::data_dir().map(|mut p| {
            p.push("habit_tracker");
            p
        }),
        // 3. User's config directory
        dirs::config_dir().map(|mut p| {
            p.push("habit_tracker");
            p
        }),
        // 4. Current working directory (last resort)
        std::env::current_dir().ok().map(|mut p| {
            p.push(".habit_tracker");
            p
        }),
    ];

    for potential_path in potential_paths.iter().flatten() {
        // Try to create the directory
        if let Ok(()) = std::fs::create_dir_all(potential_path) {
            // Test if we can write to this directory
            let test_file = potential_path.join(".test_write");
            if std::fs::write(&test_file, "test").is_ok() {
                let _ = std::fs::remove_file(&test_file); // Clean up test file
                let mut db_path = potential_path.clone();
                db_path.push("habits.db");
                return Ok(db_path);
            }
        }
    }

    // Ultimate fallback: use a temporary directory
    let mut temp_path = std::env::temp_dir();
    temp_path.push("habit_tracker");
    std::fs::create_dir_all(&temp_path)?;
    temp_path.push("habits.db");

    tracing::warn!("Using temporary directory for database: {}", temp_path.display());
    Ok(temp_path)
}

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
        Some(path) => {
            // Validate and prepare the provided path
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            path
        }
        None => {
            // Use a robust default path strategy
            get_default_database_path()?
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