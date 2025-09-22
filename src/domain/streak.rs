/// Streak calculation and tracking functionality
/// 
/// This module defines the Streak struct that holds calculated streak information
/// for a habit, and provides methods for calculating streaks from habit entries.

use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, Utc, Datelike};
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

        match frequency {
            Frequency::Daily => {
                let mut checking_date = today;

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
            Frequency::Weekly(times_per_week) => {
                // For weekly habits, check completion within weekly periods
                let mut current_week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
                let mut consecutive_weeks = 0;

                for week_offset in 0..52 { // Check up to a year
                    let week_start = current_week_start - chrono::Duration::weeks(week_offset);
                    let week_end = week_start + chrono::Duration::days(6);

                    let completions_this_week = entries.iter()
                        .filter(|e| e.completed_at >= week_start && e.completed_at <= week_end)
                        .count();

                    if completions_this_week >= *times_per_week as usize {
                        consecutive_weeks += 1;
                    } else {
                        break;
                    }
                }

                current_streak = consecutive_weeks;
            }
            Frequency::Weekdays => {
                // Check consecutive weekdays (Mon-Fri)
                let mut checking_date = today;

                // Start from today or the last weekday if today is weekend
                if matches!(checking_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                    // Go back to Friday
                    while matches!(checking_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                        checking_date = checking_date - chrono::Duration::days(1);
                    }
                }

                // If today is a weekday and not completed, start from yesterday
                if !matches!(today.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                    let has_today = entries.iter().any(|e| e.completed_at == today);
                    if !has_today {
                        checking_date = checking_date - chrono::Duration::days(1);
                        // Skip to previous weekday if needed
                        while matches!(checking_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                            checking_date = checking_date - chrono::Duration::days(1);
                        }
                    }
                }

                for _ in 0..365 { // Prevent infinite loop
                    if matches!(checking_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                        // Skip weekends
                        checking_date = checking_date - chrono::Duration::days(1);
                        continue;
                    }

                    if entries.iter().any(|e| e.completed_at == checking_date) {
                        current_streak += 1;
                    } else {
                        break;
                    }

                    checking_date = checking_date - chrono::Duration::days(1);
                }
            }
            Frequency::Weekends => {
                // Check consecutive weekends (Sat-Sun)
                let mut checking_date = today;

                // Start from today or the last weekend day if today is weekday
                if !matches!(checking_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                    // Go back to Sunday
                    while !matches!(checking_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                        checking_date = checking_date - chrono::Duration::days(1);
                    }
                }

                // If today is a weekend and not completed, start from yesterday
                if matches!(today.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                    let has_today = entries.iter().any(|e| e.completed_at == today);
                    if !has_today {
                        checking_date = checking_date - chrono::Duration::days(1);
                        // Skip to previous weekend if needed
                        while !matches!(checking_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                            checking_date = checking_date - chrono::Duration::days(1);
                        }
                    }
                }

                for _ in 0..365 { // Prevent infinite loop
                    if !matches!(checking_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                        // Skip weekdays
                        checking_date = checking_date - chrono::Duration::days(1);
                        continue;
                    }

                    if entries.iter().any(|e| e.completed_at == checking_date) {
                        current_streak += 1;
                    } else {
                        break;
                    }

                    checking_date = checking_date - chrono::Duration::days(1);
                }
            }
            Frequency::Custom(weekdays) => {
                // Check consecutive occurrences of custom weekdays
                let mut checking_date = today;

                // Start from today if it's a target day, otherwise find the most recent target day
                if !weekdays.contains(&checking_date.weekday()) {
                    for _ in 0..7 { // Look back at most a week
                        checking_date = checking_date - chrono::Duration::days(1);
                        if weekdays.contains(&checking_date.weekday()) {
                            break;
                        }
                    }
                }

                // If today is a target day and not completed, start from previous occurrence
                if weekdays.contains(&today.weekday()) {
                    let has_today = entries.iter().any(|e| e.completed_at == today);
                    if !has_today {
                        checking_date = checking_date - chrono::Duration::days(1);
                        // Find previous target day
                        for _ in 0..7 {
                            if weekdays.contains(&checking_date.weekday()) {
                                break;
                            }
                            checking_date = checking_date - chrono::Duration::days(1);
                        }
                    }
                }

                for _ in 0..365 { // Prevent infinite loop
                    if !weekdays.contains(&checking_date.weekday()) {
                        // Skip non-target days
                        checking_date = checking_date - chrono::Duration::days(1);
                        continue;
                    }

                    if entries.iter().any(|e| e.completed_at == checking_date) {
                        current_streak += 1;
                    } else {
                        break;
                    }

                    checking_date = checking_date - chrono::Duration::days(1);
                }
            }
            Frequency::Interval(days_interval) => {
                // For interval habits (e.g., every 3 days), check consecutive intervals
                let mut checking_date = today;

                // Find the most recent expected date based on interval
                // This is simplified - ideally we'd track the habit's start date
                let latest_entry = entries.first().unwrap();
                let days_since_latest = (today - latest_entry.completed_at).num_days();

                // Start from today if it should be done today, otherwise from the last expected date
                if days_since_latest % (*days_interval as i64) == 0 && !entries.iter().any(|e| e.completed_at == today) {
                    checking_date = today - chrono::Duration::days(*days_interval as i64);
                } else {
                    checking_date = today;
                    // Find the most recent valid interval date
                    for _ in 0..(*days_interval as i64) {
                        if entries.iter().any(|e| e.completed_at == checking_date) {
                            break;
                        }
                        checking_date = checking_date - chrono::Duration::days(1);
                    }
                }

                // Count consecutive intervals
                for _ in 0..365 { // Prevent infinite loop
                    if entries.iter().any(|e| e.completed_at == checking_date) {
                        current_streak += 1;
                        checking_date = checking_date - chrono::Duration::days(*days_interval as i64);
                    } else {
                        break;
                    }
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

        // Sort entries by completion date (oldest first for longest streak calculation)
        let mut sorted_entries = entries.to_vec();
        sorted_entries.sort_by(|a, b| a.completed_at.cmp(&b.completed_at));

        let mut longest_streak = 0;

        match frequency {
            Frequency::Daily => {
                let mut current_streak = 1;
                let mut last_date = sorted_entries[0].completed_at;

                for entry in sorted_entries.iter().skip(1) {
                    let days_diff = (entry.completed_at - last_date).num_days();

                    if days_diff == 1 {
                        // Consecutive day
                        current_streak += 1;
                    } else {
                        // Streak broken, record if it's the longest
                        longest_streak = longest_streak.max(current_streak);
                        current_streak = 1;
                    }

                    last_date = entry.completed_at;
                }

                // Don't forget the last streak
                longest_streak = longest_streak.max(current_streak);
            }
            Frequency::Weekly(times_per_week) => {
                // Group entries by week and find longest consecutive weeks meeting the requirement
                let mut weeks_map: std::collections::HashMap<i32, u32> = std::collections::HashMap::new();

                for entry in &sorted_entries {
                    let week_number = entry.completed_at.iso_week().week() as i32;
                    let year = entry.completed_at.year();
                    let week_key = year * 100 + week_number; // Unique key for year+week

                    *weeks_map.entry(week_key).or_insert(0) += 1;
                }

                // Sort weeks by week_key
                let mut week_counts: Vec<(i32, u32)> = weeks_map.into_iter().collect();
                week_counts.sort_by_key(|&(week_key, _)| week_key);

                let mut current_streak = 0;
                let mut last_week_key = None;

                for (week_key, count) in week_counts {
                    if count >= *times_per_week as u32 {
                        if let Some(last_key) = last_week_key {
                            // Check if this week is consecutive to the last qualifying week
                            if week_key == last_key + 1 || (week_key > last_key + 50 && week_key < last_key + 60) {
                                // Handle year boundary (week 52/53 -> week 1)
                                current_streak += 1;
                            } else {
                                longest_streak = longest_streak.max(current_streak);
                                current_streak = 1;
                            }
                        } else {
                            current_streak = 1;
                        }
                        last_week_key = Some(week_key);
                    } else {
                        longest_streak = longest_streak.max(current_streak);
                        current_streak = 0;
                        last_week_key = None;
                    }
                }

                longest_streak = longest_streak.max(current_streak);
            }
            Frequency::Weekdays => {
                let mut current_streak = 1;
                let mut last_date = sorted_entries[0].completed_at;

                for entry in sorted_entries.iter().skip(1) {
                    let mut expected_date = last_date + chrono::Duration::days(1);

                    // Skip weekends
                    while matches!(expected_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                        expected_date = expected_date + chrono::Duration::days(1);
                    }

                    if entry.completed_at == expected_date {
                        current_streak += 1;
                    } else {
                        longest_streak = longest_streak.max(current_streak);
                        current_streak = 1;
                    }

                    last_date = entry.completed_at;
                }

                longest_streak = longest_streak.max(current_streak);
            }
            Frequency::Weekends => {
                let mut current_streak = 1;
                let mut last_date = sorted_entries[0].completed_at;

                for entry in sorted_entries.iter().skip(1) {
                    let mut expected_date = last_date + chrono::Duration::days(1);

                    // Skip weekdays
                    while !matches!(expected_date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                        expected_date = expected_date + chrono::Duration::days(1);
                    }

                    if entry.completed_at == expected_date {
                        current_streak += 1;
                    } else {
                        longest_streak = longest_streak.max(current_streak);
                        current_streak = 1;
                    }

                    last_date = entry.completed_at;
                }

                longest_streak = longest_streak.max(current_streak);
            }
            Frequency::Custom(weekdays) => {
                let mut current_streak = 1;
                let mut last_date = sorted_entries[0].completed_at;

                for entry in sorted_entries.iter().skip(1) {
                    let mut expected_date = last_date + chrono::Duration::days(1);

                    // Find next target weekday
                    while !weekdays.contains(&expected_date.weekday()) {
                        expected_date = expected_date + chrono::Duration::days(1);
                        // Prevent infinite loop if no valid weekdays are specified
                        if (expected_date - last_date).num_days() > 7 {
                            break;
                        }
                    }

                    if entry.completed_at == expected_date {
                        current_streak += 1;
                    } else {
                        longest_streak = longest_streak.max(current_streak);
                        current_streak = 1;
                    }

                    last_date = entry.completed_at;
                }

                longest_streak = longest_streak.max(current_streak);
            }
            Frequency::Interval(days_interval) => {
                // For interval habits, check consecutive intervals
                let mut current_streak = 1;
                let mut last_date = sorted_entries[0].completed_at;

                for entry in sorted_entries.iter().skip(1) {
                    let expected_date = last_date + chrono::Duration::days(*days_interval as i64);

                    if entry.completed_at == expected_date {
                        current_streak += 1;
                    } else {
                        longest_streak = longest_streak.max(current_streak);
                        current_streak = 1;
                    }

                    last_date = entry.completed_at;
                }

                longest_streak = longest_streak.max(current_streak);
            }
        }

        longest_streak
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