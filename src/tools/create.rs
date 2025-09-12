/// Tool for creating new habits
/// 
/// This module implements the habit_create MCP tool.

use serde::{Deserialize, Serialize};
use crate::domain::{Habit, Category, Frequency};
use crate::storage::{StorageError, HabitStorage};

/// Parameters for creating a new habit
#[derive(Debug, Deserialize)]
pub struct CreateHabitParams {
    pub name: String,
    pub description: Option<String>,
    pub category: String, // We'll parse this to Category enum
    pub frequency: String, // We'll parse this to Frequency enum
    pub target_value: Option<u32>,
    pub unit: Option<String>,
}

/// Response from creating a habit
#[derive(Debug, Serialize)]
pub struct CreateHabitResponse {
    pub success: bool,
    pub habit_id: Option<String>,
    pub message: String,
}

/// Create a new habit using the provided storage
pub fn create_habit<S: HabitStorage>(
    storage: &S,
    params: CreateHabitParams,
) -> Result<CreateHabitResponse, StorageError> {
    // Validate input parameters
    if params.name.trim().is_empty() {
        return Err(StorageError::Query(
            rusqlite::Error::InvalidColumnType(0, "Habit name cannot be empty".to_string(), rusqlite::types::Type::Text)
        ));
    }
    
    if params.name.len() > 100 {
        return Err(StorageError::Query(
            rusqlite::Error::InvalidColumnType(0, "Habit name too long (max 100 characters)".to_string(), rusqlite::types::Type::Text)
        ));
    }
    
    // Parse and validate category
    let category = match params.category.trim().to_lowercase().as_str() {
        "health" => Category::Health,
        "productivity" => Category::Productivity,
        "social" => Category::Social,
        "creative" => Category::Creative,
        "mindfulness" => Category::Mindfulness,
        "financial" => Category::Financial,
        "household" => Category::Household,
        "personal" => Category::Personal,
        custom if custom.starts_with("custom:") => {
            let name = custom.strip_prefix("custom:").unwrap().trim();
            if name.is_empty() {
                return Err(StorageError::Query(
                    rusqlite::Error::InvalidColumnType(0, "Custom category name cannot be empty".to_string(), rusqlite::types::Type::Text)
                ));
            }
            Category::Custom(name.to_string())
        },
        _ => {
            return Err(StorageError::Query(
                rusqlite::Error::InvalidColumnType(0, 
                    format!("Invalid category '{}'. Valid options: health, productivity, social, creative, mindfulness, financial, household, personal, or custom:name", params.category),
                    rusqlite::types::Type::Text
                )
            ));
        }
    };
    
    // Parse and validate frequency
    let frequency = match params.frequency.trim().to_lowercase().as_str() {
        "daily" => Frequency::Daily,
        "weekdays" => Frequency::Weekdays,
        "weekends" => Frequency::Weekends,
        "weekly" => Frequency::Weekly(3), // Default to 3 times per week
        "custom" => Frequency::Custom(vec![chrono::Weekday::Mon]), // Default to Monday
        _ => {
            return Err(StorageError::Query(
                rusqlite::Error::InvalidColumnType(0, 
                    format!("Invalid frequency '{}'. Valid options: daily, weekdays, weekends, weekly, custom", params.frequency),
                    rusqlite::types::Type::Text
                )
            ));
        }
    };
    
    // Create the habit
    let habit = Habit::new(
        params.name.clone(),
        params.description,
        category,
        frequency,
        params.target_value,
        params.unit,
    ).map_err(|e| StorageError::Query(
        rusqlite::Error::InvalidColumnType(0, e.to_string(), rusqlite::types::Type::Text)
    ))?;
    
    let habit_id = habit.id.to_string();
    
    // Save to storage
    storage.create_habit(&habit)?;
    
    Ok(CreateHabitResponse {
        success: true,
        habit_id: Some(habit_id),
        message: format!("✅ Created habit '{}'! Ready to start your streak!", params.name),
    })
}