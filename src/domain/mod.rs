/// Domain module containing core business logic and data types
/// 
/// This module defines the core entities (Habit, HabitEntry, Streak) and their
/// validation rules. These types represent the fundamental concepts in our
/// habit tracking system.

pub mod habit;
pub mod entry;  
pub mod streak;
pub mod types;

// Re-export public types for easy access
pub use habit::*;
pub use entry::*;
pub use streak::*;
pub use types::*;

use thiserror::Error;

/// Errors that can occur during domain operations
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Invalid habit name: {0}")]
    InvalidHabitName(String),
    
    #[error("Invalid frequency: {0}")]
    InvalidFrequency(String),
    
    #[error("Invalid date: {0}")]
    InvalidDate(String),
    
    #[error("Invalid value: {message}")]
    InvalidValue { message: String },
}