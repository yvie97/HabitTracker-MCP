/// Habit entity and related functionality
/// 
/// This module defines the core Habit struct that represents a user's habit
/// they want to track, along with validation and builder patterns.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::domain::{Category, Frequency, HabitId, DomainError};

/// A habit represents something the user wants to do regularly
/// 
/// This is the core entity in our system. Each habit has a name, category,
/// frequency (how often it should be done), and optional target values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Habit {
    /// Unique identifier for this habit
    pub id: HabitId,
    /// Display name (e.g., "Morning Run", "Read for 30min")
    pub name: String,
    /// Optional detailed description
    pub description: Option<String>,
    /// Category for organization (health, productivity, etc.)
    pub category: Category,
    /// How often this habit should be performed
    pub frequency: Frequency,
    /// Optional numeric target (e.g., 30 for "30 minutes")
    pub target_value: Option<u32>,
    /// Unit for the target value (e.g., "minutes", "pages", "reps")
    pub unit: Option<String>,
    /// When this habit was created
    pub created_at: DateTime<Utc>,
    /// Whether this habit is currently active (can be paused)
    pub is_active: bool,
}

impl Habit {
    /// Create a new habit with validation
    /// 
    /// This is the main constructor that validates all fields and returns
    /// an error if any validation fails.
    pub fn new(
        name: String,
        description: Option<String>,
        category: Category,
        frequency: Frequency,
        target_value: Option<u32>,
        unit: Option<String>,
    ) -> Result<Self, DomainError> {
        // Validate the habit data
        Self::validate_name(&name)?;
        Self::validate_description(&description)?;
        frequency.validate()?;
        Self::validate_target_and_unit(&target_value, &unit)?;
        
        Ok(Self {
            id: HabitId::new(),
            name,
            description,
            category,
            frequency,
            target_value,
            unit,
            created_at: Utc::now(),
            is_active: true,
        })
    }
    
    /// Create a habit from existing data (used when loading from database)
    /// 
    /// This constructor assumes data is already validated and is mainly used
    /// by the storage layer when loading habits from the database.
    pub fn from_existing(
        id: HabitId,
        name: String,
        description: Option<String>,
        category: Category,
        frequency: Frequency,
        target_value: Option<u32>,
        unit: Option<String>,
        created_at: DateTime<Utc>,
        is_active: bool,
    ) -> Self {
        Self {
            id,
            name,
            description,
            category,
            frequency,
            target_value,
            unit,
            created_at,
            is_active,
        }
    }
    
    /// Update the habit's properties with validation
    /// 
    /// This allows modifying an existing habit while ensuring all validation
    /// rules are still met.
    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<Option<String>>,
        frequency: Option<Frequency>,
        target_value: Option<Option<u32>>,
        unit: Option<Option<String>>,
        is_active: Option<bool>,
    ) -> Result<(), DomainError> {
        // Validate new values before applying them
        if let Some(ref new_name) = name {
            Self::validate_name(new_name)?;
        }
        
        if let Some(ref new_desc) = description {
            Self::validate_description(new_desc)?;
        }
        
        if let Some(ref new_freq) = frequency {
            new_freq.validate()?;
        }
        
        // For target/unit updates, we need to validate them together
        let new_target = target_value.unwrap_or(self.target_value);
        let new_unit = unit.clone().unwrap_or(self.unit.clone());
        Self::validate_target_and_unit(&new_target, &new_unit)?;
        
        // Apply updates
        if let Some(new_name) = name {
            self.name = new_name;
        }
        if let Some(new_description) = description {
            self.description = new_description;
        }
        if let Some(new_frequency) = frequency {
            self.frequency = new_frequency;
        }
        if let Some(new_target_value) = target_value {
            self.target_value = new_target_value;
        }
        if let Some(new_unit) = unit {
            self.unit = new_unit;
        }
        if let Some(new_is_active) = is_active {
            self.is_active = new_is_active;
        }
        
        Ok(())
    }
    
    /// Check if this habit has a numeric target
    pub fn has_target(&self) -> bool {
        self.target_value.is_some()
    }
    
    /// Get a display string for the target (e.g., "30 minutes")
    pub fn target_display(&self) -> Option<String> {
        match (self.target_value, &self.unit) {
            (Some(value), Some(unit)) => Some(format!("{} {}", value, unit)),
            (Some(value), None) => Some(value.to_string()),
            _ => None,
        }
    }
    
    // Validation helper methods
    
    /// Validate habit name according to business rules
    fn validate_name(name: &str) -> Result<(), DomainError> {
        let trimmed = name.trim();
        
        if trimmed.is_empty() {
            return Err(DomainError::InvalidHabitName(
                "Habit name cannot be empty".to_string()
            ));
        }
        
        if trimmed.len() > 100 {
            return Err(DomainError::InvalidHabitName(
                "Habit name cannot be longer than 100 characters".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate optional description
    fn validate_description(description: &Option<String>) -> Result<(), DomainError> {
        if let Some(desc) = description {
            if desc.len() > 500 {
                return Err(DomainError::Validation {
                    message: "Description cannot be longer than 500 characters".to_string()
                });
            }
        }
        Ok(())
    }
    
    /// Validate target value and unit together
    fn validate_target_and_unit(
        target_value: &Option<u32>,
        unit: &Option<String>,
    ) -> Result<(), DomainError> {
        match (target_value, unit) {
            (Some(value), _) => {
                if *value == 0 {
                    return Err(DomainError::InvalidValue {
                        message: "Target value must be greater than 0".to_string()
                    });
                }
                if *value > 10000 {
                    return Err(DomainError::InvalidValue {
                        message: "Target value cannot exceed 10000".to_string()
                    });
                }
            }
            _ => {}
        }
        
        if let Some(unit_str) = unit {
            let trimmed = unit_str.trim();
            if trimmed.is_empty() {
                return Err(DomainError::InvalidValue {
                    message: "Unit cannot be empty if specified".to_string()
                });
            }
            if trimmed.len() > 20 {
                return Err(DomainError::InvalidValue {
                    message: "Unit cannot be longer than 20 characters".to_string()
                });
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;
    
    #[test]
    fn test_create_valid_habit() {
        let habit = Habit::new(
            "Morning Run".to_string(),
            Some("30-minute jog around the neighborhood".to_string()),
            Category::Health,
            Frequency::Daily,
            Some(30),
            Some("minutes".to_string()),
        );
        
        assert!(habit.is_ok());
        let habit = habit.unwrap();
        assert_eq!(habit.name, "Morning Run");
        assert_eq!(habit.category, Category::Health);
        assert!(habit.is_active);
        assert!(habit.has_target());
        assert_eq!(habit.target_display(), Some("30 minutes".to_string()));
    }
    
    #[test]
    fn test_invalid_habit_name() {
        let result = Habit::new(
            "".to_string(), // Empty name should fail
            None,
            Category::Health,
            Frequency::Daily,
            None,
            None,
        );
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_invalid_target_value() {
        let result = Habit::new(
            "Test Habit".to_string(),
            None,
            Category::Health,
            Frequency::Daily,
            Some(0), // Zero target should fail
            Some("minutes".to_string()),
        );
        
        assert!(result.is_err());
    }
}