/// Tool for listing all habits
/// 
/// This module implements the habit_list MCP tool.

use serde::{Deserialize, Serialize};
use crate::domain::{Category, Frequency};
use crate::storage::{StorageError, HabitStorage};
use crate::analytics::AnalyticsEngine;
use chrono::Weekday;

/// Parameters for listing habits
#[derive(Debug, Deserialize)]
pub struct ListHabitsParams {
    pub category: Option<String>,
    pub active_only: Option<bool>,
    pub sort_by: Option<String>, // "name", "streak", "created_at", "completion_rate"
}

/// Information about a habit in the list
#[derive(Debug, Serialize)]
pub struct HabitSummary {
    pub habit_id: String,
    pub name: String,
    pub category: String,
    pub frequency: String,
    pub current_streak: u32,
    pub completion_rate: f64,
    pub total_completions: u32,
    pub is_active: bool,
}

/// Summary statistics for all habits
#[derive(Debug, Serialize)]
pub struct HabitListSummary {
    pub total_habits: u32,
    pub active_habits: u32,
    pub avg_completion_rate: f64,
}

/// Response from listing habits
#[derive(Debug, Serialize)]
pub struct ListHabitsResponse {
    pub habits: Vec<HabitSummary>,
    pub summary: HabitListSummary,
}

/// List habits using the provided storage
pub fn list_habits<S: HabitStorage>(
    storage: &S,
    params: ListHabitsParams,
) -> Result<ListHabitsResponse, StorageError> {
    // Parse category filter
    let category_filter = params.category.and_then(|cat_str| {
        match cat_str.as_str() {
            "health" => Some(Category::Health),
            "productivity" => Some(Category::Productivity),
            "social" => Some(Category::Social),
            "creative" => Some(Category::Creative),
            "mindfulness" => Some(Category::Mindfulness),
            "financial" => Some(Category::Financial),
            "household" => Some(Category::Household),
            "personal" => Some(Category::Personal),
            _ => None,
        }
    });
    
    let active_only = params.active_only.unwrap_or(true);
    
    // Get habits from storage
    let habits = storage.list_habits(category_filter, active_only)?;

    let analytics = AnalyticsEngine::new();

    // Convert to response format with actual data
    let mut habit_summaries: Vec<HabitSummary> = Vec::new();

    for habit in habits {
        // Get streak data for this habit
        let streak = match storage.get_streak(&habit.id) {
            Ok(streak) => streak,
            Err(_) => {
                // If no streak data exists, get entries and calculate
                let entries = storage.get_entries_for_habit(&habit.id, None)?;
                analytics.calculate_habit_streak(&habit, &entries)
            }
        };

        let habit_summary = HabitSummary {
            habit_id: habit.id.to_string(),
            name: habit.name,
            category: match habit.category {
                Category::Health => "health".to_string(),
                Category::Productivity => "productivity".to_string(),
                Category::Social => "social".to_string(),
                Category::Creative => "creative".to_string(),
                Category::Mindfulness => "mindfulness".to_string(),
                Category::Financial => "financial".to_string(),
                Category::Household => "household".to_string(),
                Category::Personal => "personal".to_string(),
                Category::Custom(name) => name,
            },
            frequency: frequency_to_display_string(&habit.frequency),
            current_streak: streak.current_streak,
            completion_rate: streak.completion_rate,
            total_completions: streak.total_completions,
            is_active: habit.is_active,
        };

        habit_summaries.push(habit_summary);
    }

    // Sort by requested criteria
    let sort_by = params.sort_by.as_deref().unwrap_or("name");
    habit_summaries.sort_by(|a, b| {
        match sort_by {
            "streak" => b.current_streak.cmp(&a.current_streak),
            "completion_rate" => b.completion_rate.partial_cmp(&a.completion_rate).unwrap_or(std::cmp::Ordering::Equal),
            "total_completions" => b.total_completions.cmp(&a.total_completions),
            "created_at" => a.name.cmp(&b.name), // Fallback to name since we don't have created_at in summary
            _ => a.name.cmp(&b.name), // Default to name sorting
        }
    });
    
    let total_habits = habit_summaries.len() as u32;
    let active_habits = habit_summaries.iter()
        .filter(|h| h.is_active)
        .count() as u32;
    let avg_completion_rate = if habit_summaries.is_empty() {
        0.0
    } else {
        habit_summaries.iter()
            .map(|h| h.completion_rate)
            .sum::<f64>() / habit_summaries.len() as f64
    };
    
    Ok(ListHabitsResponse {
        habits: habit_summaries,
        summary: HabitListSummary {
            total_habits,
            active_habits,
            avg_completion_rate,
        },
    })
}

/// Convert frequency to a human-readable display string
fn frequency_to_display_string(frequency: &Frequency) -> String {
    match frequency {
        Frequency::Daily => "Daily".to_string(),
        Frequency::Weekly(times) => {
            if *times == 1 {
                "Weekly".to_string()
            } else {
                format!("{} times per week", times)
            }
        }
        Frequency::Weekdays => "Weekdays".to_string(),
        Frequency::Weekends => "Weekends".to_string(),
        Frequency::Custom(days) => {
            let day_names: Vec<String> = days.iter().map(|day| {
                match day {
                    Weekday::Mon => "Mon",
                    Weekday::Tue => "Tue",
                    Weekday::Wed => "Wed",
                    Weekday::Thu => "Thu",
                    Weekday::Fri => "Fri",
                    Weekday::Sat => "Sat",
                    Weekday::Sun => "Sun",
                }.to_string()
            }).collect();
            day_names.join(", ")
        }
        Frequency::Interval(days) => {
            format!("Every {} day{}", days, if *days == 1 { "" } else { "s" })
        }
    }
}