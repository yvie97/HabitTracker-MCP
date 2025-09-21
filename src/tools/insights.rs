/// Tool for providing habit insights and recommendations
///
/// This module implements the habit_insights MCP tool that analyzes
/// habit data to provide useful insights and personalized recommendations.

use crate::analytics::{AnalyticsEngine, InsightsParams, InsightsResponse};
use crate::storage::{StorageError, HabitStorage};


/// Analyze habits and generate insights
pub fn get_habit_insights<S: HabitStorage>(
    storage: &S,
    params: InsightsParams,
) -> Result<InsightsResponse, StorageError> {
    let analytics = AnalyticsEngine::new();
    analytics.get_habit_insights(storage, params)
}

