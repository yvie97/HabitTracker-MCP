/// Database migration management
/// 
/// This module handles creating and updating the SQLite database schema.
/// It ensures the database has all the required tables and indexes.

use rusqlite::{Connection};
use crate::storage::StorageError;

/// Current database schema version
/// 
/// Increment this when you add new migrations
const CURRENT_VERSION: i32 = 1;

/// Initialize the database schema
/// 
/// This creates all required tables and indexes if they don't exist.
/// It also sets up the version tracking for future migrations.
pub fn initialize_database(conn: &Connection) -> Result<(), StorageError> {
    // Create version tracking table first
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        [],
    )?;
    
    // Check current version
    let current_version = get_current_version(conn)?;
    
    // Run migrations if needed
    if current_version < CURRENT_VERSION {
        run_migrations(conn, current_version)?;
        set_version(conn, CURRENT_VERSION)?;
    }
    
    Ok(())
}

/// Get the current database schema version
fn get_current_version(conn: &Connection) -> Result<i32, StorageError> {
    let version = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get::<_, i32>(0)
        })
        .unwrap_or(0); // Default to version 0 if no version record exists
    
    Ok(version)
}

/// Set the database schema version
fn set_version(conn: &Connection, version: i32) -> Result<(), StorageError> {
    conn.execute("DELETE FROM schema_version", [])?;
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        [version],
    )?;
    Ok(())
}

/// Run database migrations from the current version to the latest
fn run_migrations(conn: &Connection, from_version: i32) -> Result<(), StorageError> {
    if from_version < 1 {
        migration_v1(conn)?;
    }
    
    // Future migrations would go here:
    // if from_version < 2 {
    //     migration_v2(conn)?;
    // }
    
    Ok(())
}

/// Migration to version 1: Create initial tables
/// 
/// This creates the core tables for habits, entries, and streaks
fn migration_v1(conn: &Connection) -> Result<(), StorageError> {
    // Create habits table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habits (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            category TEXT NOT NULL,
            frequency_type TEXT NOT NULL,
            frequency_data TEXT,
            target_value INTEGER,
            unit TEXT,
            created_at TEXT NOT NULL,
            is_active BOOLEAN DEFAULT TRUE
        )",
        [],
    )?;
    
    // Create habit_entries table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habit_entries (
            id TEXT PRIMARY KEY,
            habit_id TEXT NOT NULL,
            logged_at TEXT NOT NULL,
            completed_at TEXT NOT NULL,
            value INTEGER,
            intensity INTEGER,
            notes TEXT,
            FOREIGN KEY (habit_id) REFERENCES habits (id)
        )",
        [],
    )?;
    
    // Create habit_streaks table (cached calculations)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habit_streaks (
            habit_id TEXT PRIMARY KEY,
            current_streak INTEGER NOT NULL DEFAULT 0,
            longest_streak INTEGER NOT NULL DEFAULT 0,
            last_completed TEXT,
            total_completions INTEGER NOT NULL DEFAULT 0,
            completion_rate REAL NOT NULL DEFAULT 0.0,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (habit_id) REFERENCES habits (id)
        )",
        [],
    )?;
    
    // Create indexes for better query performance
    create_indexes_v1(conn)?;
    
    tracing::info!("Applied migration v1: Created initial database schema");
    Ok(())
}

/// Create database indexes for version 1
fn create_indexes_v1(conn: &Connection) -> Result<(), StorageError> {
    // Index for finding entries by habit and date (most common query)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_habit_entries_habit_completed 
         ON habit_entries (habit_id, completed_at)",
        [],
    )?;
    
    // Index for finding entries by date (for analytics)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_habit_entries_completed_at 
         ON habit_entries (completed_at)",
        [],
    )?;
    
    // Index for filtering habits by category
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_habits_category 
         ON habits (category)",
        [],
    )?;
    
    // Index for filtering active habits
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_habits_active 
         ON habits (is_active)",
        [],
    )?;
    
    // Unique constraint to prevent duplicate entries for same habit/date
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_habit_entries_unique 
         ON habit_entries (habit_id, completed_at)",
        [],
    )?;
    
    tracing::info!("Created database indexes for v1");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
    #[test]
    fn test_initialize_database() {
        let conn = Connection::open_in_memory().unwrap();
        
        // Should succeed on a fresh database
        let result = initialize_database(&conn);
        assert!(result.is_ok());
        
        // Should succeed when called again (idempotent)
        let result = initialize_database(&conn);
        assert!(result.is_ok());
        
        // Verify tables were created
        let table_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('habits', 'habit_entries', 'habit_streaks')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        
        assert_eq!(table_count, 3);
    }
    
    #[test]
    fn test_version_tracking() {
        let conn = Connection::open_in_memory().unwrap();
        
        // Initialize should set version to current
        initialize_database(&conn).unwrap();
        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }
}