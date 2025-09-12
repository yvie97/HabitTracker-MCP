/// Core types and enums used throughout the domain layer
/// 
/// This module defines the fundamental types like Category, Frequency, and ID types
/// that are used by Habit, HabitEntry, and other domain entities.

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Weekday, Datelike};
use uuid::Uuid;

/// Unique identifier for a habit
/// 
/// This is a wrapper around UUID to provide type safety - you can't accidentally
/// pass a habit ID where an entry ID is expected.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HabitId(pub Uuid);

impl HabitId {
    /// Generate a new random habit ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Create a habit ID from a string (useful for database loading)
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// Unique identifier for a habit entry
/// 
/// Similar to HabitId but for individual habit completion records
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntryId(pub Uuid);

impl EntryId {
    /// Generate a new random entry ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Create an entry ID from a string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// Categories for organizing habits into different life areas
/// 
/// This helps users organize their habits and enables category-based analytics.
/// Users can also define custom categories beyond the predefined ones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    /// Health-related habits (exercise, diet, sleep)
    Health,
    /// Work and learning habits (studying, skill building)
    Productivity,
    /// Relationship and communication habits
    Social,
    /// Creative pursuits (art, writing, music)
    Creative,
    /// Meditation, reflection, gratitude practices
    Mindfulness,
    /// Money management and financial habits
    Financial,
    /// Home maintenance and organization
    Household,
    /// Personal growth and self-care
    Personal,
    /// User-defined category with custom name
    Custom(String),
}

impl Category {
    /// Get the display name for this category
    pub fn display_name(&self) -> &str {
        match self {
            Category::Health => "Health",
            Category::Productivity => "Productivity",
            Category::Social => "Social",
            Category::Creative => "Creative",
            Category::Mindfulness => "Mindfulness",
            Category::Financial => "Financial",
            Category::Household => "Household",
            Category::Personal => "Personal",
            Category::Custom(name) => name,
        }
    }
}

/// How often a habit should be performed
/// 
/// This supports various scheduling patterns from daily habits to complex
/// weekly schedules. The frequency affects how streaks are calculated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frequency {
    /// Every single day
    Daily,
    /// A specific number of times per week (1-7)
    Weekly(u8),
    /// Monday through Friday only
    Weekdays,
    /// Saturday and Sunday only
    Weekends,
    /// Specific days of the week (e.g., Monday, Wednesday, Friday)
    Custom(Vec<Weekday>),
    /// Every N days (e.g., every 3 days)
    Interval(u32),
}

impl Frequency {
    /// Validate that a frequency value is reasonable
    pub fn validate(&self) -> Result<(), crate::domain::DomainError> {
        match self {
            Frequency::Weekly(times) => {
                if *times == 0 || *times > 7 {
                    return Err(crate::domain::DomainError::InvalidFrequency(
                        format!("Weekly frequency must be 1-7, got {}", times)
                    ));
                }
            }
            Frequency::Custom(days) => {
                if days.is_empty() {
                    return Err(crate::domain::DomainError::InvalidFrequency(
                        "Custom frequency must specify at least one day".to_string()
                    ));
                }
                if days.len() > 7 {
                    return Err(crate::domain::DomainError::InvalidFrequency(
                        "Custom frequency cannot have more than 7 days".to_string()
                    ));
                }
            }
            Frequency::Interval(days) => {
                if *days == 0 {
                    return Err(crate::domain::DomainError::InvalidFrequency(
                        "Interval must be at least 1 day".to_string()
                    ));
                }
                if *days > 365 {
                    return Err(crate::domain::DomainError::InvalidFrequency(
                        "Interval cannot be longer than 365 days".to_string()
                    ));
                }
            }
            _ => {} // Daily, Weekdays, Weekends are always valid
        }
        Ok(())
    }
    
    /// Check if this frequency expects the habit to be done on a given date
    pub fn is_scheduled_for_date(&self, date: NaiveDate) -> bool {
        match self {
            Frequency::Daily => true,
            Frequency::Weekdays => {
                let weekday = date.weekday();
                !matches!(weekday, Weekday::Sat | Weekday::Sun)
            }
            Frequency::Weekends => {
                let weekday = date.weekday();
                matches!(weekday, Weekday::Sat | Weekday::Sun)
            }
            Frequency::Custom(days) => {
                let weekday = date.weekday();
                days.contains(&weekday)
            }
            Frequency::Weekly(_) => {
                // For weekly habits, we consider them "scheduled" every day
                // but the streak logic will consider the weekly target
                true
            }
            Frequency::Interval(_) => {
                // For interval habits, this depends on when the habit was started
                // This is more complex and would need the habit creation date
                // For now, we'll return true and handle this in streak calculation
                true
            }
        }
    }
}