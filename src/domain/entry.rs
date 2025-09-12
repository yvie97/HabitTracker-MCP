/// HabitEntry entity for tracking habit completions
/// 
/// This module defines the HabitEntry struct that represents a single instance
/// of completing a habit on a specific day, with optional values and notes.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};
use crate::domain::{EntryId, HabitId, DomainError};

/// A record of completing a habit on a specific day
/// 
/// Each time a user logs a habit completion, we create a HabitEntry.
/// This includes when it was logged, which day it was for, and optional
/// details like intensity ratings and notes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HabitEntry {
    /// Unique identifier for this entry
    pub id: EntryId,
    /// Which habit this entry is for
    pub habit_id: HabitId,
    /// When this entry was created/logged
    pub logged_at: DateTime<Utc>,
    /// Which day this completion was for (can be different from logged_at)
    pub completed_at: NaiveDate,
    /// Actual amount achieved (if habit has a target)
    pub value: Option<u32>,
    /// Subjective intensity rating from 1-10
    pub intensity: Option<u8>,
    /// User's notes about this completion
    pub notes: Option<String>,
}

impl HabitEntry {
    /// Create a new habit entry with validation
    /// 
    /// This validates all the input data and creates a new entry.
    /// The logged_at timestamp is set to the current time.
    pub fn new(
        habit_id: HabitId,
        completed_at: NaiveDate,
        value: Option<u32>,
        intensity: Option<u8>,
        notes: Option<String>,
    ) -> Result<Self, DomainError> {
        // Validate the entry data
        Self::validate_completed_at(&completed_at)?;
        Self::validate_value(&value)?;
        Self::validate_intensity(&intensity)?;
        Self::validate_notes(&notes)?;
        
        Ok(Self {
            id: EntryId::new(),
            habit_id,
            logged_at: Utc::now(),
            completed_at,
            value,
            intensity,
            notes,
        })
    }
    
    /// Create an entry from existing data (used when loading from database)
    /// 
    /// This constructor assumes data is already validated and is mainly used
    /// by the storage layer when loading entries from the database.
    pub fn from_existing(
        id: EntryId,
        habit_id: HabitId,
        logged_at: DateTime<Utc>,
        completed_at: NaiveDate,
        value: Option<u32>,
        intensity: Option<u8>,
        notes: Option<String>,
    ) -> Self {
        Self {
            id,
            habit_id,
            logged_at,
            completed_at,
            value,
            intensity,
            notes,
        }
    }
    
    /// Check if this entry has a numeric value
    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }
    
    /// Check if this entry has an intensity rating
    pub fn has_intensity(&self) -> bool {
        self.intensity.is_some()
    }
    
    /// Check if this entry has notes
    pub fn has_notes(&self) -> bool {
        self.notes.is_some() && !self.notes.as_ref().unwrap().trim().is_empty()
    }
    
    // Validation helper methods
    
    /// Validate that the completed_at date is not in the future
    fn validate_completed_at(date: &NaiveDate) -> Result<(), DomainError> {
        let today = Utc::now().naive_utc().date();
        
        if *date > today {
            return Err(DomainError::InvalidDate(
                "Cannot log habits for future dates".to_string()
            ));
        }
        
        // Don't allow entries too far in the past (more than 1 year)
        let one_year_ago = today - chrono::Duration::days(365);
        if *date < one_year_ago {
            return Err(DomainError::InvalidDate(
                "Cannot log habits more than 1 year in the past".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate the optional value field
    fn validate_value(value: &Option<u32>) -> Result<(), DomainError> {
        if let Some(val) = value {
            if *val > 100000 {
                return Err(DomainError::InvalidValue {
                    message: "Value cannot exceed 100000".to_string()
                });
            }
        }
        Ok(())
    }
    
    /// Validate the optional intensity rating (1-10)
    fn validate_intensity(intensity: &Option<u8>) -> Result<(), DomainError> {
        if let Some(rating) = intensity {
            if *rating < 1 || *rating > 10 {
                return Err(DomainError::InvalidValue {
                    message: "Intensity must be between 1 and 10".to_string()
                });
            }
        }
        Ok(())
    }
    
    /// Validate the optional notes field
    fn validate_notes(notes: &Option<String>) -> Result<(), DomainError> {
        if let Some(note_text) = notes {
            if note_text.len() > 500 {
                return Err(DomainError::InvalidValue {
                    message: "Notes cannot be longer than 500 characters".to_string()
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_create_valid_entry() {
        let habit_id = HabitId::new();
        let today = Utc::now().naive_utc().date();
        
        let entry = HabitEntry::new(
            habit_id.clone(),
            today,
            Some(30),
            Some(8),
            Some("Felt great today!".to_string()),
        );
        
        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert_eq!(entry.habit_id, habit_id);
        assert_eq!(entry.completed_at, today);
        assert_eq!(entry.value, Some(30));
        assert_eq!(entry.intensity, Some(8));
        assert!(entry.has_value());
        assert!(entry.has_intensity());
        assert!(entry.has_notes());
    }
    
    #[test]
    fn test_future_date_invalid() {
        let habit_id = HabitId::new();
        let future_date = Utc::now().naive_utc().date() + chrono::Duration::days(1);
        
        let result = HabitEntry::new(
            habit_id,
            future_date,
            None,
            None,
            None,
        );
        
        assert!(result.is_err());
    }
}