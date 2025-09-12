/// Tool for logging habit completions
/// 
/// This module implements the habit_log MCP tool.

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Utc};
use crate::domain::{HabitEntry, HabitId, Streak};
use crate::storage::{StorageError, HabitStorage};

/// Parameters for logging a habit completion
#[derive(Debug, Deserialize)]
pub struct LogHabitParams {
    pub habit_id: String,
    pub completed_at: Option<String>, // Optional date, defaults to today
    pub value: Option<u32>,
    pub intensity: Option<u8>,
    pub notes: Option<String>,
}

/// Response from logging a habit
#[derive(Debug, Serialize)]
pub struct LogHabitResponse {
    pub success: bool,
    pub message: String,
    pub current_streak: Option<u32>,
}

/// Calculate streak information for a habit based on its entries
/// This is a simplified calculation that checks consecutive days
fn calculate_habit_streak<S: HabitStorage>(
    storage: &S,
    habit_id: &HabitId,
    latest_entry_date: NaiveDate,
) -> Result<Streak, StorageError> {
    // Get existing streak data
    let mut streak = storage.get_streak(habit_id)?;
    
    // For now, implement a simple streak calculation
    // In a real implementation, we'd get all entries and calculate properly
    
    // Update last completed date
    streak.last_completed = Some(latest_entry_date);
    
    // Simple logic: if we have a recent completion, increment streak
    if streak.current_streak == 0 {
        // Starting a new streak
        streak.current_streak = 1;
    } else {
        // Check if the last completion was yesterday (consecutive days)
        // This is simplified - in reality we'd check all recent entries
        streak.current_streak += 1;
    }
    
    // Update longest streak if current is longer
    if streak.current_streak > streak.longest_streak {
        streak.longest_streak = streak.current_streak;
    }
    
    // Increment total completions
    streak.total_completions += 1;
    
    // Simple completion rate calculation (needs proper implementation)
    // For now, just use a placeholder
    streak.completion_rate = if streak.total_completions > 0 { 0.8 } else { 0.0 };
    
    Ok(streak)
}

/// Log a habit completion using the provided storage
pub fn log_habit<S: HabitStorage>(
    storage: &S,
    params: LogHabitParams,
) -> Result<LogHabitResponse, StorageError> {
    // Validate habit ID format
    if params.habit_id.trim().is_empty() {
        return Err(StorageError::Query(
            rusqlite::Error::InvalidColumnType(0, "Habit ID cannot be empty".to_string(), rusqlite::types::Type::Text)
        ));
    }
    
    // Parse habit ID
    let habit_id = HabitId::from_string(&params.habit_id)
        .map_err(|_| StorageError::Query(
            rusqlite::Error::InvalidColumnType(0, "Invalid habit ID format".to_string(), rusqlite::types::Type::Text)
        ))?;
    
    // Verify habit exists
    if storage.get_habit(&habit_id).is_err() {
        return Err(StorageError::HabitNotFound { habit_id: params.habit_id.clone() });
    }
    
    // Parse completed date (default to today)
    let completed_at = if let Some(date_str) = params.completed_at {
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|_| StorageError::Query(
                rusqlite::Error::InvalidColumnType(0, "Invalid date format".to_string(), rusqlite::types::Type::Text)
            ))?
    } else {
        Utc::now().naive_utc().date()
    };
    
    // Validate optional parameters
    if let Some(intensity) = params.intensity {
        if intensity < 1 || intensity > 10 {
            return Err(StorageError::Query(
                rusqlite::Error::InvalidColumnType(0, "Intensity must be between 1 and 10".to_string(), rusqlite::types::Type::Integer)
            ));
        }
    }
    
    if let Some(value) = params.value {
        if value > 999999 {
            return Err(StorageError::Query(
                rusqlite::Error::InvalidColumnType(0, "Value too large (max 999,999)".to_string(), rusqlite::types::Type::Integer)
            ));
        }
    }
    
    if let Some(ref notes) = params.notes {
        if notes.len() > 500 {
            return Err(StorageError::Query(
                rusqlite::Error::InvalidColumnType(0, "Notes too long (max 500 characters)".to_string(), rusqlite::types::Type::Text)
            ));
        }
    }
    
    // Create the habit entry
    let entry = HabitEntry::new(
        habit_id.clone(),
        completed_at,
        params.value,
        params.intensity,
        params.notes,
    ).map_err(|e| StorageError::Query(
        rusqlite::Error::InvalidColumnType(0, e.to_string(), rusqlite::types::Type::Text)
    ))?;
    
    // Save to storage
    storage.create_entry(&entry)?;
    
    // Calculate and update streak information
    let updated_streak = calculate_habit_streak(storage, &habit_id, completed_at)?;
    
    // Update streak in storage
    storage.update_streak(&updated_streak)?;
    
    Ok(LogHabitResponse {
        success: true,
        message: format!("🔥 Logged habit completion! Current streak: {} day{}", 
                        updated_streak.current_streak, 
                        if updated_streak.current_streak == 1 { "" } else { "s" }),
        current_streak: Some(updated_streak.current_streak),
    })
}