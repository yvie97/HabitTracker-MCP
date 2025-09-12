/// Storage layer for persisting habit data
/// 
/// This module handles all database operations using SQLite. It provides
/// a clean interface for storing and retrieving habits, entries, and streaks.

pub mod sqlite;
pub mod migrations;

// Re-export the main storage types
pub use sqlite::*;

use thiserror::Error;
use crate::domain::{Habit, HabitEntry, Streak, HabitId, Category};

/// Errors that can occur during storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection error: {0}")]
    Connection(String),
    
    #[error("Database query error: {0}")]
    Query(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Habit not found: {habit_id}")]
    HabitNotFound { habit_id: String },
    
    #[error("Entry not found: {entry_id}")]
    EntryNotFound { entry_id: String },
    
    #[error("Duplicate entry: habit {habit_id} already logged for date {date}")]
    DuplicateEntry { habit_id: String, date: String },
    
    #[error("Migration error: {0}")]
    Migration(String),
}

/// Trait defining the storage interface for habits
/// 
/// This trait allows us to potentially swap out SQLite for other databases
/// in the future while keeping the same interface.
pub trait HabitStorage {
    /// Create a new habit
    fn create_habit(&self, habit: &Habit) -> Result<(), StorageError>;
    
    /// Get a habit by ID
    fn get_habit(&self, habit_id: &HabitId) -> Result<Habit, StorageError>;
    
    /// Update an existing habit
    fn update_habit(&self, habit: &Habit) -> Result<(), StorageError>;
    
    /// Delete a habit (soft delete - mark as inactive)
    fn delete_habit(&self, habit_id: &HabitId) -> Result<(), StorageError>;
    
    /// List habits with optional filtering
    fn list_habits(
        &self,
        category: Option<Category>,
        active_only: bool,
    ) -> Result<Vec<Habit>, StorageError>;
    
    /// Create a new habit entry
    fn create_entry(&self, entry: &HabitEntry) -> Result<(), StorageError>;
    
    /// Get entries for a specific habit
    fn get_entries_for_habit(
        &self,
        habit_id: &HabitId,
        limit: Option<u32>,
    ) -> Result<Vec<HabitEntry>, StorageError>;
    
    /// Get all entries within a date range
    fn get_entries_by_date_range(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<HabitEntry>, StorageError>;
    
    /// Update or create streak data for a habit
    fn update_streak(&self, streak: &Streak) -> Result<(), StorageError>;
    
    /// Get streak data for a habit
    fn get_streak(&self, habit_id: &HabitId) -> Result<Streak, StorageError>;
    
    /// Get streak data for all habits
    fn get_all_streaks(&self) -> Result<Vec<Streak>, StorageError>;
}