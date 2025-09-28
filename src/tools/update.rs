/// Tool for updating existing habits
///
/// This module implements the habit_update MCP tool to modify
/// existing habit properties like name, frequency, targets, etc.

use serde::{Deserialize, Serialize};
use crate::domain::{Frequency, HabitId};
use crate::storage::{StorageError, HabitStorage};

/// Parameters for updating an existing habit
#[derive(Debug, Deserialize)]
pub struct UpdateHabitParams {
    pub habit_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub target_value: Option<u32>,
    pub unit: Option<String>,
    pub is_active: Option<bool>,
}

/// Response from updating a habit
#[derive(Debug, Serialize)]
pub struct UpdateHabitResponse {
    pub success: bool,
    pub message: String,
}

/// Update an existing habit using the provided storage
pub fn update_habit<S: HabitStorage>(
    storage: &S,
    params: UpdateHabitParams,
) -> Result<UpdateHabitResponse, StorageError> {
    // Parse and validate habit ID
    let habit_id = HabitId::from_string(&params.habit_id)
        .map_err(|_| StorageError::HabitNotFound { habit_id: params.habit_id.clone() })?;

    // Fetch the existing habit
    let mut habit = storage.get_habit(&habit_id)?;

    // Parse frequency if provided
    let frequency = if let Some(freq_str) = params.frequency {
        Some(parse_frequency(&freq_str)?)
    } else {
        None
    };

    // Validate and apply updates
    habit.update(
        params.name,
        params.description.map(Some), // Wrap in Option for the method signature
        frequency,
        params.target_value.map(Some), // Wrap in Option for the method signature
        params.unit.map(Some), // Wrap in Option for the method signature
        params.is_active,
    ).map_err(|e| StorageError::Query(
        rusqlite::Error::InvalidColumnType(0, e.to_string(), rusqlite::types::Type::Text)
    ))?;

    // Save the updated habit
    storage.update_habit(&habit)?;

    // Generate appropriate success message
    let message = if let Some(false) = params.is_active {
        format!("⏸️ Paused habit '{}'", habit.name)
    } else if let Some(true) = params.is_active {
        format!("▶️ Reactivated habit '{}'", habit.name)
    } else {
        format!("✅ Updated habit '{}'", habit.name)
    };

    Ok(UpdateHabitResponse {
        success: true,
        message,
    })
}

/// Parse frequency string into Frequency enum
fn parse_frequency(freq_str: &str) -> Result<Frequency, StorageError> {
    match freq_str.trim().to_lowercase().as_str() {
        "daily" => Ok(Frequency::Daily),
        "weekdays" => Ok(Frequency::Weekdays),
        "weekends" => Ok(Frequency::Weekends),
        "weekly" => Ok(Frequency::Weekly(3)), // Default to 3 times per week
        "custom" => Ok(Frequency::Custom(vec![chrono::Weekday::Mon])), // Default to Monday
        _ => Err(StorageError::Query(
            rusqlite::Error::InvalidColumnType(0,
                format!("Invalid frequency '{}'. Valid options: daily, weekdays, weekends, weekly, custom", freq_str),
                rusqlite::types::Type::Text
            )
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Habit, Category, Frequency};
    use crate::storage::sqlite::SqliteStorage;
    use tempfile::tempdir;

    #[test]
    fn test_update_habit_name() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = SqliteStorage::new(db_path.to_str().unwrap()).unwrap();

        // Create a test habit
        let habit = Habit::new(
            "Old Name".to_string(),
            None,
            Category::Health,
            Frequency::Daily,
            None,
            None,
        ).unwrap();

        let habit_id = habit.id.to_string();
        storage.create_habit(&habit).unwrap();

        // Update the habit name
        let params = UpdateHabitParams {
            habit_id: habit_id.clone(),
            name: Some("New Name".to_string()),
            description: None,
            frequency: None,
            target_value: None,
            unit: None,
            is_active: None,
        };

        let result = update_habit(&storage, params);
        assert!(result.is_ok());

        // Verify the update
        let updated_habit = storage.get_habit(&HabitId::from_string(&habit_id).unwrap()).unwrap();
        assert_eq!(updated_habit.name, "New Name");
    }

    #[test]
    fn test_update_habit_active_status() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = SqliteStorage::new(db_path.to_str().unwrap()).unwrap();

        // Create a test habit
        let habit = Habit::new(
            "Test Habit".to_string(),
            None,
            Category::Health,
            Frequency::Daily,
            None,
            None,
        ).unwrap();

        let habit_id = habit.id.to_string();
        storage.create_habit(&habit).unwrap();

        // Pause the habit
        let params = UpdateHabitParams {
            habit_id: habit_id.clone(),
            name: None,
            description: None,
            frequency: None,
            target_value: None,
            unit: None,
            is_active: Some(false),
        };

        let result = update_habit(&storage, params);
        assert!(result.is_ok());
        assert!(result.unwrap().message.contains("Paused"));

        // Verify the update
        let updated_habit = storage.get_habit(&HabitId::from_string(&habit_id).unwrap()).unwrap();
        assert!(!updated_habit.is_active);
    }

    #[test]
    fn test_update_nonexistent_habit() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = SqliteStorage::new(db_path.to_str().unwrap()).unwrap();

        let params = UpdateHabitParams {
            habit_id: "nonexistent_id".to_string(),
            name: Some("New Name".to_string()),
            description: None,
            frequency: None,
            target_value: None,
            unit: None,
            is_active: None,
        };

        let result = update_habit(&storage, params);
        assert!(result.is_err());
    }
}