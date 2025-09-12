/// Tool for providing habit insights and recommendations
/// 
/// This module implements the habit_insights MCP tool that analyzes
/// habit data to provide useful insights and personalized recommendations.

use serde::{Deserialize, Serialize};
use chrono::{Utc};
use crate::domain::{HabitId, Category};
use crate::storage::{StorageError, HabitStorage};

/// Parameters for getting habit insights
#[derive(Debug, Deserialize)]
pub struct InsightsParams {
    pub habit_id: Option<String>, // If omitted, provides insights for all habits
    pub time_period: Option<String>, // "week", "month", "quarter", "year"
    pub insight_type: Option<String>, // "performance", "recommendations", "patterns"
}

/// Individual insight with analysis
#[derive(Debug, Serialize)]
pub struct Insight {
    pub title: String,
    pub message: String,
    pub insight_type: String, // "success", "warning", "recommendation", "pattern"
    pub confidence: f64, // 0.0 to 1.0
    pub data: Option<serde_json::Value>, // Additional structured data
}

/// Response containing habit insights
#[derive(Debug, Serialize)]
pub struct InsightsResponse {
    pub insights: Vec<Insight>,
    pub summary: String,
    pub message: String,
    pub time_period: String,
    pub generated_at: String,
}

/// Analyze habits and generate insights
pub fn get_habit_insights<S: HabitStorage>(
    storage: &S,
    params: InsightsParams,
) -> Result<InsightsResponse, StorageError> {
    let time_period = params.time_period.unwrap_or("month".to_string());
    let insight_type = params.insight_type.unwrap_or("all".to_string());
    
    let mut insights = Vec::new();
    
    if let Some(habit_id_str) = params.habit_id {
        // Generate insights for specific habit
        let habit_id = HabitId::from_string(&habit_id_str)
            .map_err(|_| StorageError::HabitNotFound { habit_id: habit_id_str.clone() })?;
        
        insights.extend(generate_single_habit_insights(storage, &habit_id, &time_period)?);
    } else {
        // Generate insights for all habits
        insights.extend(generate_overall_insights(storage, &time_period)?);
    }
    
    // Filter by insight type if specified
    if insight_type != "all" {
        insights.retain(|insight| insight.insight_type == insight_type);
    }
    
    let summary = if insights.is_empty() {
        "No specific insights available yet. Keep tracking your habits to build more data!".to_string()
    } else {
        let success_count = insights.iter().filter(|i| i.insight_type == "success").count();
        let recommendation_count = insights.iter().filter(|i| i.insight_type == "recommendation").count();
        
        format!("Generated {} insights: {} successes, {} recommendations", 
                insights.len(), success_count, recommendation_count)
    };
    
    let message = format!("📊 **Habit Insights Report** ({})\n\n{}\n\n{}", 
                         time_period.to_uppercase(),
                         summary,
                         insights.iter()
                             .map(|i| format!("{} **{}**\n   {}", 
                                             get_insight_emoji(&i.insight_type),
                                             i.title, 
                                             i.message))
                             .collect::<Vec<_>>()
                             .join("\n\n"));
    
    Ok(InsightsResponse {
        insights,
        summary,
        message,
        time_period,
        generated_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    })
}

/// Generate insights for a single habit
fn generate_single_habit_insights<S: HabitStorage>(
    storage: &S,
    habit_id: &HabitId,
    _time_period: &str,
) -> Result<Vec<Insight>, StorageError> {
    let mut insights = Vec::new();
    
    // Get streak data for the habit
    let streak = storage.get_streak(habit_id)?;
    
    // Streak analysis
    if streak.current_streak >= 7 {
        insights.push(Insight {
            title: "Great Consistency!".to_string(),
            message: format!("You've maintained this habit for {} days straight. That's excellent dedication!", streak.current_streak),
            insight_type: "success".to_string(),
            confidence: 0.9,
            data: Some(serde_json::json!({
                "current_streak": streak.current_streak,
                "streak_milestone": get_streak_milestone(streak.current_streak)
            })),
        });
    } else if streak.current_streak == 0 && streak.longest_streak > 0 {
        insights.push(Insight {
            title: "Time to Restart".to_string(),
            message: format!("You've had a {} day streak before - you can do it again! Start with just completing the habit once today.", streak.longest_streak),
            insight_type: "recommendation".to_string(),
            confidence: 0.8,
            data: Some(serde_json::json!({
                "longest_streak": streak.longest_streak,
                "current_streak": streak.current_streak
            })),
        });
    }
    
    // Completion rate analysis
    if streak.completion_rate >= 0.8 {
        insights.push(Insight {
            title: "High Performer".to_string(),
            message: format!("You're completing this habit {:.0}% of the time. This is excellent performance!", streak.completion_rate * 100.0),
            insight_type: "success".to_string(),
            confidence: 0.9,
            data: Some(serde_json::json!({
                "completion_rate": streak.completion_rate,
                "performance_level": "excellent"
            })),
        });
    } else if streak.completion_rate >= 0.6 {
        insights.push(Insight {
            title: "Good Progress".to_string(),
            message: format!("You're at {:.0}% completion rate. Try to identify what helps you succeed and do more of that!", streak.completion_rate * 100.0),
            insight_type: "recommendation".to_string(),
            confidence: 0.7,
            data: Some(serde_json::json!({
                "completion_rate": streak.completion_rate,
                "performance_level": "good"
            })),
        });
    } else if streak.total_completions > 0 {
        insights.push(Insight {
            title: "Room for Improvement".to_string(),
            message: format!("Your completion rate is {:.0}%. Consider setting smaller, more achievable goals to build momentum.", streak.completion_rate * 100.0),
            insight_type: "recommendation".to_string(),
            confidence: 0.8,
            data: Some(serde_json::json!({
                "completion_rate": streak.completion_rate,
                "performance_level": "needs_improvement",
                "suggestion": "break_down_habit"
            })),
        });
    }
    
    Ok(insights)
}

/// Generate overall insights across all habits
fn generate_overall_insights<S: HabitStorage>(
    storage: &S,
    _time_period: &str,
) -> Result<Vec<Insight>, StorageError> {
    let mut insights = Vec::new();
    
    // Get all habits
    let habits = storage.list_habits(None, true)?;
    
    if habits.is_empty() {
        insights.push(Insight {
            title: "Get Started".to_string(),
            message: "Welcome to habit tracking! Start by creating your first habit. Choose something small and achievable.".to_string(),
            insight_type: "recommendation".to_string(),
            confidence: 1.0,
            data: Some(serde_json::json!({
                "action": "create_first_habit",
                "suggestions": ["drink_water", "read_5_minutes", "walk_10_minutes"]
            })),
        });
        return Ok(insights);
    }
    
    // Analyze habit portfolio
    let mut active_streaks = 0;
    let mut total_streak_days = 0;
    let mut category_counts = std::collections::HashMap::new();
    let mut completion_rates = Vec::new();
    
    for habit in &habits {
        if let Ok(streak) = storage.get_streak(&habit.id) {
            if streak.current_streak > 0 {
                active_streaks += 1;
                total_streak_days += streak.current_streak;
            }
            completion_rates.push(streak.completion_rate);
        }
        
        let category_name = match &habit.category {
            Category::Health => "Health",
            Category::Productivity => "Productivity", 
            Category::Social => "Social",
            Category::Creative => "Creative",
            Category::Mindfulness => "Mindfulness",
            Category::Financial => "Financial",
            Category::Household => "Household",
            Category::Personal => "Personal",
            Category::Custom(name) => name,
        };
        *category_counts.entry(category_name.to_string()).or_insert(0) += 1;
    }
    
    // Portfolio analysis
    if active_streaks > 0 {
        insights.push(Insight {
            title: "Momentum Building".to_string(),
            message: format!("You have {} active streak{} totaling {} days! This shows great consistency across your habit portfolio.", 
                           active_streaks, 
                           if active_streaks == 1 { "" } else { "s" },
                           total_streak_days),
            insight_type: "success".to_string(),
            confidence: 0.9,
            data: Some(serde_json::json!({
                "active_streaks": active_streaks,
                "total_streak_days": total_streak_days,
                "total_habits": habits.len()
            })),
        });
    }
    
    // Category diversity insight
    if category_counts.len() >= 3 {
        insights.push(Insight {
            title: "Well-Rounded Growth".to_string(),
            message: format!("You're working on {} different life areas: {}. This balanced approach supports overall life improvement!", 
                           category_counts.len(),
                           category_counts.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")),
            insight_type: "success".to_string(),
            confidence: 0.8,
            data: Some(serde_json::json!({
                "categories": category_counts,
                "diversity_score": category_counts.len() as f64 / 8.0 // Max 8 categories
            })),
        });
    } else if habits.len() > 3 {
        insights.push(Insight {
            title: "Consider Diversifying".to_string(),
            message: "Most of your habits are in similar categories. Try adding habits from different life areas for more balanced growth.".to_string(),
            insight_type: "recommendation".to_string(),
            confidence: 0.7,
            data: Some(serde_json::json!({
                "current_categories": category_counts,
                "suggested_categories": ["Health", "Mindfulness", "Social", "Creative"]
            })),
        });
    }
    
    // Overall performance insight
    if !completion_rates.is_empty() {
        let avg_completion = completion_rates.iter().sum::<f64>() / completion_rates.len() as f64;
        if avg_completion >= 0.7 {
            insights.push(Insight {
                title: "Excellent Overall Performance".to_string(),
                message: format!("Your average completion rate across all habits is {:.0}%. You're building strong, sustainable routines!", avg_completion * 100.0),
                insight_type: "success".to_string(),
                confidence: 0.9,
                data: Some(serde_json::json!({
                    "average_completion_rate": avg_completion,
                    "performance_tier": "excellent"
                })),
            });
        }
    }
    
    // Habit load recommendation
    if habits.len() > 5 && active_streaks < habits.len() / 2 {
        insights.push(Insight {
            title: "Focus Strategy".to_string(),
            message: format!("You have {} habits but only {} active streaks. Consider focusing on 2-3 core habits to build stronger foundations.", 
                           habits.len(), active_streaks),
            insight_type: "recommendation".to_string(),
            confidence: 0.8,
            data: Some(serde_json::json!({
                "total_habits": habits.len(),
                "active_streaks": active_streaks,
                "recommended_focus": 3,
                "strategy": "focus_and_build"
            })),
        });
    }
    
    Ok(insights)
}

/// Get appropriate emoji for insight type
fn get_insight_emoji(insight_type: &str) -> &'static str {
    match insight_type {
        "success" => "🎉",
        "warning" => "⚠️",
        "recommendation" => "💡",
        "pattern" => "📈",
        _ => "📊",
    }
}

/// Get milestone description for streak length
fn get_streak_milestone(streak: u32) -> &'static str {
    match streak {
        1..=6 => "building_momentum",
        7..=13 => "one_week_strong", 
        14..=20 => "two_weeks_solid",
        21..=29 => "three_weeks_excellent",
        30..=59 => "one_month_amazing",
        60..=89 => "two_months_incredible",
        90.. => "habit_master",
        _ => "just_started",
    }
}