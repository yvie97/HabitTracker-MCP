/// Streak calculation and tracking functionality
/// 
/// This module defines the Streak struct that holds calculated streak information
/// for a habit, and provides methods for calculating streaks from habit entries.

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Utc};
use crate::domain::{HabitId, HabitEntry, Frequency};

/// Calculated streak information for a habit
/// 
/// This struct holds all the streak-related statistics for a habit.
/// Streaks are calculated based on the habit's frequency and completion history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Streak {
    /// Which habit this streak data is for
    pub habit_id: HabitId,
    /// Current consecutive days/periods completed
    pub current_streak: u32,
    /// Best streak ever achieved for this habit
    pub longest_streak: u32,
    /// When the habit was last completed (None if never completed)
    pub last_completed: Option<NaiveDate>,
    /// Total number of times this habit has been completed
    pub total_completions: u32,
    /// Completion rate since habit creation (0.0 to 1.0)
    pub completion_rate: f64,
}

impl Streak {
    /// Create a new streak record with zero values
    /// 
    /// This creates an empty streak record for a new habit that hasn't
    /// been completed yet.
    pub fn new(habit_id: HabitId) -> Self {
        Self {
            habit_id,
            current_streak: 0,
            longest_streak: 0,
            last_completed: None,
            total_completions: 0,
            completion_rate: 0.0,
        }
    }
    
    /// Create a streak from existing data (used when loading from database)
    pub fn from_existing(
        habit_id: HabitId,
        current_streak: u32,
        longest_streak: u32,
        last_completed: Option<NaiveDate>,
        total_completions: u32,
        completion_rate: f64,
    ) -> Self {
        Self {
            habit_id,
            current_streak,
            longest_streak,
            last_completed,
            total_completions,
            completion_rate,
        }
    }
    
    /// Calculate streak information from a list of habit entries
    /// 
    /// This is the main method that analyzes all entries for a habit and
    /// calculates the current streak, longest streak, and completion rate.
    pub fn calculate_from_entries(
        habit_id: HabitId,
        entries: &[HabitEntry],
        frequency: &Frequency,
        habit_created_at: NaiveDate,
    ) -> Self {
        if entries.is_empty() {
            return Self::new(habit_id);
        }
        
        // Sort entries by completion date (newest first)
        let mut sorted_entries = entries.to_vec();
        sorted_entries.sort_by(|a, b| b.completed_at.cmp(&a.completed_at));
        
        let total_completions = entries.len() as u32;
        let last_completed = sorted_entries.first().map(|e| e.completed_at);
        
        // Calculate current streak
        let current_streak = Self::calculate_current_streak(&sorted_entries, frequency);
        
        // Calculate longest streak
        let longest_streak = Self::calculate_longest_streak(&sorted_entries, frequency);
        
        // Calculate completion rate
        let completion_rate = Self::calculate_completion_rate(
            &sorted_entries,
            frequency,
            habit_created_at,
        );
        
        Self {
            habit_id,
            current_streak,
            longest_streak: longest_streak.max(current_streak),
            last_completed,
            total_completions,
            completion_rate,
        }
    }
    
    /// Check if the habit is currently "on track" based on frequency
    pub fn is_on_track(&self, frequency: &Frequency) -> bool {
        let today = Utc::now().naive_utc().date();
        
        match self.last_completed {
            None => false, // Never completed
            Some(last_date) => {
                match frequency {
                    Frequency::Daily => {
                        // On track if completed today or yesterday
                        let days_since = (today - last_date).num_days();
                        days_since <= 1
                    }
                    Frequency::Weekdays => {
                        // More complex logic for weekdays only
                        let days_since = (today - last_date).num_days();
                        days_since <= 3 // Allow for weekends
                    }
                    Frequency::Weekly(_) => {
                        // On track if completed within the last week
                        let days_since = (today - last_date).num_days();
                        days_since <= 7
                    }
                    _ => {
                        // For other frequencies, use a generous 3-day window
                        let days_since = (today - last_date).num_days();
                        days_since <= 3
                    }
                }
            }
        }
    }
    
    /// Get a motivational message based on current streak status
    pub fn motivational_message(&self) -> String {
        match self.current_streak {
            0 => "Ready to start your streak! Every journey begins with a single step.".to_string(),
            1 => "Great start! One day down, keep the momentum going.".to_string(),
            2..=6 => format!("Nice work! {} days in a row. You're building a strong habit.", self.current_streak),
            7..=13 => format!("Excellent! {} days strong. You're in the groove now!", self.current_streak),
            14..=29 => format!("Amazing! {} days straight. This is becoming second nature.", self.current_streak),
            30..=99 => format!("Incredible! {} days of consistency. You're a habit master!", self.current_streak),
            _ => format!("Legendary! {} days of unwavering commitment. You're an inspiration!", self.current_streak),
        }
    }
    
    // Private helper methods for streak calculation
    
    /// Calculate the current active streak
    fn calculate_current_streak(entries: &[HabitEntry], frequency: &Frequency) -> u32 {
        if entries.is_empty() {
            return 0;
        }
        
        let today = Utc::now().naive_utc().date();
        let mut current_streak = 0;
        let mut checking_date = today;
        
        // For daily habits, check each day backwards
        match frequency {
            Frequency::Daily => {
                // Check if we need to start from yesterday (if today isn't completed yet)
                let has_today = entries.iter().any(|e| e.completed_at == today);
                if !has_today {
                    checking_date = today - chrono::Duration::days(1);
                }
                
                // Count consecutive days backwards
                for _ in 0..365 { // Prevent infinite loop
                    if entries.iter().any(|e| e.completed_at == checking_date) {
                        current_streak += 1;
                        checking_date = checking_date - chrono::Duration::days(1);
                    } else {
                        break;
                    }
                }
            }
            _ => {
                // For other frequencies, use a simpler approach
                // This is a simplified version - real implementation would be more complex
                let latest_entry = entries.first().unwrap();
                let days_since = (today - latest_entry.completed_at).num_days();
                
                if days_since <= 1 {
                    current_streak = 1;
                    // TODO: Implement proper streak calculation for other frequencies
                }
            }
        }
        
        current_streak
    }
    
    /// Calculate the longest streak achieved
    fn calculate_longest_streak(entries: &[HabitEntry], frequency: &Frequency) -> u32 {
        if entries.is_empty() {
            return 0;
        }
        
        // For simplicity, we'll use the same logic as current streak
        // but check all possible starting points
        // TODO: Implement proper longest streak calculation
        Self::calculate_current_streak(entries, frequency)
    }
    
    /// Calculate completion rate since habit creation
    fn calculate_completion_rate(
        entries: &[HabitEntry],
        frequency: &Frequency,
        created_at: NaiveDate,
    ) -> f64 {
        if entries.is_empty() {
            return 0.0;
        }
        
        let today = Utc::now().naive_utc().date();
        let days_since_creation = (today - created_at).num_days() + 1; // Include creation day
        
        let expected_completions = match frequency {
            Frequency::Daily => days_since_creation as f64,
            Frequency::Weekly(times) => {
                let weeks = days_since_creation as f64 / 7.0;
                weeks * (*times as f64)
            }
            Frequency::Weekdays => {
                // Approximate: 5 days per week
                let weeks = days_since_creation as f64 / 7.0;
                weeks * 5.0
            }
            Frequency::Weekends => {
                // Approximate: 2 days per week
                let weeks = days_since_creation as f64 / 7.0;
                weeks * 2.0
            }
            _ => days_since_creation as f64, // Fallback to daily
        };
        
        if expected_completions <= 0.0 {
            return 0.0;
        }
        
        let actual_completions = entries.len() as f64;
        (actual_completions / expected_completions).min(1.0) // Cap at 100%
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EntryId;
    use chrono::{DateTime, Utc};
    
    #[test]
    fn test_new_streak() {
        let habit_id = HabitId::new();
        let streak = Streak::new(habit_id.clone());
        
        assert_eq!(streak.habit_id, habit_id);
        assert_eq!(streak.current_streak, 0);
        assert_eq!(streak.longest_streak, 0);
        assert_eq!(streak.last_completed, None);
        assert_eq!(streak.total_completions, 0);
        assert_eq!(streak.completion_rate, 0.0);
    }
    
    #[test]
    fn test_motivational_messages() {
        let habit_id = HabitId::new();
        let mut streak = Streak::new(habit_id);
        
        assert!(streak.motivational_message().contains("Ready to start"));
        
        streak.current_streak = 1;
        assert!(streak.motivational_message().contains("Great start"));
        
        streak.current_streak = 7;
        assert!(streak.motivational_message().contains("Excellent"));
        
        streak.current_streak = 100;
        assert!(streak.motivational_message().contains("Legendary"));
    }
    
    #[test]
    fn test_is_on_track_daily() {
        let habit_id = HabitId::new();
        let today = Utc::now().naive_utc().date();
        
        let streak = Streak {
            habit_id,
            current_streak: 1,
            longest_streak: 1,
            last_completed: Some(today),
            total_completions: 1,
            completion_rate: 1.0,
        };
        
        assert!(streak.is_on_track(&Frequency::Daily));
        
        let streak_yesterday = Streak {
            habit_id: HabitId::new(),
            current_streak: 1,
            longest_streak: 1,
            last_completed: Some(today - chrono::Duration::days(1)),
            total_completions: 1,
            completion_rate: 1.0,
        };
        
        assert!(streak_yesterday.is_on_track(&Frequency::Daily));
    }
}