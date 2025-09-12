/// Tool for listing all habits
/// 
/// This module implements the habit_list MCP tool.

use serde::{Deserialize, Serialize};
use crate::domain::Category;
use crate::storage::{StorageError, HabitStorage};

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
    
    // TODO: Get streak data for each habit and sort by requested criteria
    
    // Convert to response format
    let habit_summaries: Vec<HabitSummary> = habits.into_iter().map(|habit| {
        HabitSummary {
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
            frequency: "daily".to_string(), // TODO: Convert frequency properly
            current_streak: 0, // TODO: Get actual streak data
            completion_rate: 0.0, // TODO: Calculate actual rate
            total_completions: 0, // TODO: Get actual count
            is_active: habit.is_active,
        }
    }).collect();
    
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