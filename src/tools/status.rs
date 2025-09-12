/// Tool for checking habit status and streaks
/// 
/// This module implements the habit_status MCP tool.

use serde::{Deserialize, Serialize};
use crate::domain::{HabitId};
use crate::storage::{StorageError, HabitStorage};

/// Parameters for checking habit status
#[derive(Debug, Deserialize)]
pub struct StatusParams {
    pub habit_id: Option<String>, // If omitted, returns all habits
    pub include_recent: Option<bool>,
}

/// Information about a single habit's status
#[derive(Debug, Serialize)]
pub struct HabitStatus {
    pub habit_id: String,
    pub name: String,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub completion_rate: f64,
    pub last_completed: Option<String>,
    pub status: String, // "on_track", "missed", "new", etc.
}

/// Response from checking habit status
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub habits: Vec<HabitStatus>,
    pub summary: String,
    pub message: String,
}

/// Get status for habits using the provided storage
pub fn get_habit_status<S: HabitStorage>(
    storage: &S,
    params: StatusParams,
) -> Result<StatusResponse, StorageError> {
    let habits = if let Some(habit_id_str) = params.habit_id {
        // Get status for specific habit
        let habit_id = HabitId::from_string(&habit_id_str)
            .map_err(|_| StorageError::HabitNotFound { habit_id: habit_id_str.clone() })?;
        
        // Try to get the habit - for now we'll create a simple status
        // In the future, we can implement proper get_habit
        let streak = storage.get_streak(&habit_id)?;
        
        vec![HabitStatus {
            habit_id: habit_id_str,
            name: "Habit".to_string(), // We'll need to get this from storage later
            current_streak: streak.current_streak,
            longest_streak: streak.longest_streak,
            completion_rate: streak.completion_rate,
            last_completed: streak.last_completed.map(|d| d.to_string()),
            status: if streak.current_streak > 0 { "active" } else { "inactive" }.to_string(),
        }]
    } else {
        // Get status for all habits - simplified implementation
        let all_habits = storage.list_habits(None, true)?;
        let mut habit_statuses = Vec::new();
        
        for habit in all_habits {
            let streak = storage.get_streak(&habit.id)?;
            habit_statuses.push(HabitStatus {
                habit_id: habit.id.to_string(),
                name: habit.name,
                current_streak: streak.current_streak,
                longest_streak: streak.longest_streak,
                completion_rate: streak.completion_rate,
                last_completed: streak.last_completed.map(|d| d.to_string()),
                status: if streak.current_streak > 0 { "active" } else { "inactive" }.to_string(),
            });
        }
        
        habit_statuses
    };
    
    let summary = if habits.is_empty() {
        "No habits found. Create your first habit to get started!".to_string()
    } else {
        let active_count = habits.iter().filter(|h| h.current_streak > 0).count();
        let total_count = habits.len();
        format!("📊 Status: {} of {} habits active. Total streaks: {} days", 
               active_count, total_count, 
               habits.iter().map(|h| h.current_streak).sum::<u32>())
    };
    
    let message = format!("{}\n\n{}", summary, 
        habits.iter()
            .map(|h| format!("🎯 {} ({})\n   Current streak: {} days | Best: {} days | Rate: {:.1}%{}", 
                            h.name, h.habit_id[..8].to_string() + "...", 
                            h.current_streak, h.longest_streak, 
                            h.completion_rate * 100.0,
                            if let Some(last) = &h.last_completed { 
                                format!("\n   Last completed: {}", last) 
                            } else { 
                                "".to_string() 
                            }))
            .collect::<Vec<_>>()
            .join("\n\n"));
    
    Ok(StatusResponse {
        habits,
        summary,
        message,
    })
}