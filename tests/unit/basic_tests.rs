/// Basic unit tests to verify core functionality
use habit_tracker_mcp::*;
use tempfile::NamedTempFile;

#[cfg(test)]
mod basic_unit_tests {
    use super::*;

    #[test]
    fn test_habit_creation() {
        let habit = Habit::new(
            "Test Habit".to_string(),
            Some("A test habit".to_string()),
            Category::Health,
            Frequency::Daily,
            None,
            None,
        );

        assert!(habit.is_ok());
        let habit = habit.unwrap();
        assert_eq!(habit.name, "Test Habit");
    }

    #[test]
    fn test_habit_entry_creation() {
        let habit_id = HabitId::new();
        let today = chrono::Utc::now().naive_utc().date();

        let entry = HabitEntry::new(
            habit_id.clone(),
            today,
            Some(100),
            Some(8),
            Some("Great work!".to_string()),
        );

        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert_eq!(entry.habit_id, habit_id);
        assert_eq!(entry.completed_at, today);
    }

    #[test]
    fn test_basic_enum_creation() {
        let _freq = Frequency::Daily;
        let _category = Category::Health;
        assert!(true); // Basic enum creation works
    }

    #[tokio::test]
    async fn test_server_creation() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let server = HabitTrackerServer::new(temp_file.path().to_path_buf()).await;
        assert!(server.is_ok());
    }

    #[test]
    fn test_storage_creation() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let storage = SqliteStorage::new(temp_file.path().to_path_buf());
        assert!(storage.is_ok());
    }

    #[test]
    fn test_analytics_engine_creation() {
        let _analytics = AnalyticsEngine::new();
        // If we get here without panicking, creation succeeded
        assert!(true);
    }
}