/// SQLite implementation of the habit storage interface
/// 
/// This module provides the concrete SQLite implementation for storing
/// and retrieving habit data. It handles all SQL queries and data conversion.

use std::path::PathBuf;
use rusqlite::{Connection, params};
use chrono::{NaiveDate, Utc};
use serde_json;

use crate::domain::{
    Habit, HabitEntry, Streak, HabitId, EntryId, Category
};
use crate::storage::{StorageError, HabitStorage, migrations};

/// SQLite-based storage implementation
/// 
/// This struct holds a connection to the SQLite database and implements
/// all the storage operations defined in the HabitStorage trait.
pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    /// Create a new SQLite storage instance
    /// 
    /// This opens the database file and runs any necessary migrations
    /// to ensure the schema is up to date.
    pub fn new(db_path: PathBuf) -> Result<Self, StorageError> {
        // Open the SQLite database
        let conn = Connection::open(&db_path)
            .map_err(|e| StorageError::Connection(format!("Failed to open database: {}", e)))?;
        
        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| StorageError::Connection(format!("Failed to enable foreign keys: {}", e)))?;
        
        // Initialize/migrate the database schema
        migrations::initialize_database(&conn)?;
        
        tracing::info!("SQLite storage initialized at: {:?}", db_path);
        
        Ok(Self { conn })
    }
    
    /// Helper method to convert Category enum to string for database storage
    fn category_to_string(category: &Category) -> String {
        match category {
            Category::Health => "health".to_string(),
            Category::Productivity => "productivity".to_string(),
            Category::Social => "social".to_string(),
            Category::Creative => "creative".to_string(),
            Category::Mindfulness => "mindfulness".to_string(),
            Category::Financial => "financial".to_string(),
            Category::Household => "household".to_string(),
            Category::Personal => "personal".to_string(),
            Category::Custom(name) => format!("custom:{}", name),
        }
    }
    
    /// Helper method to convert string from database to Category enum
    fn string_to_category(s: &str) -> Result<Category, StorageError> {
        match s {
            "health" => Ok(Category::Health),
            "productivity" => Ok(Category::Productivity),
            "social" => Ok(Category::Social),
            "creative" => Ok(Category::Creative),
            "mindfulness" => Ok(Category::Mindfulness),
            "financial" => Ok(Category::Financial),
            "household" => Ok(Category::Household),
            "personal" => Ok(Category::Personal),
            s if s.starts_with("custom:") => {
                let name = s.strip_prefix("custom:").unwrap().to_string();
                Ok(Category::Custom(name))
            }
            _ => Err(StorageError::Query(rusqlite::Error::InvalidColumnType(
                0, "Invalid category".to_string(), rusqlite::types::Type::Text
            ))),
        }
    }
}

impl HabitStorage for SqliteStorage {
    /// Create a new habit in the database
    fn create_habit(&self, habit: &Habit) -> Result<(), StorageError> {
        let category_str = Self::category_to_string(&habit.category);
        let frequency_json = serde_json::to_string(&habit.frequency)?;
        
        self.conn.execute(
            "INSERT INTO habits (
                id, name, description, category, frequency_type, frequency_data,
                target_value, unit, created_at, is_active
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                habit.id.to_string(),
                habit.name,
                habit.description,
                category_str,
                "json", // We're storing frequency as JSON
                frequency_json,
                habit.target_value,
                habit.unit,
                habit.created_at.to_rfc3339(),
                habit.is_active
            ],
        )?;
        
        tracing::debug!("Created habit: {} ({})", habit.name, habit.id.to_string());
        Ok(())
    }
    
    /// Get a habit by its ID
    fn get_habit(&self, habit_id: &HabitId) -> Result<Habit, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, category, frequency_data, target_value, unit, created_at, is_active 
             FROM habits WHERE id = ?1"
        )?;
        
        let result = stmt.query_row(params![habit_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let id = HabitId::from_string(&id_str).map_err(|_| {
                rusqlite::Error::InvalidColumnType(0, "Invalid UUID".to_string(), rusqlite::types::Type::Text)
            })?;
            
            let category_str: String = row.get(3)?;
            let category = Self::string_to_category(&category_str).map_err(|_| {
                rusqlite::Error::InvalidColumnType(3, "Invalid category".to_string(), rusqlite::types::Type::Text)
            })?;
            
            let frequency_json: String = row.get(4)?;
            let frequency = serde_json::from_str(&frequency_json).map_err(|_| {
                rusqlite::Error::InvalidColumnType(4, "Invalid frequency".to_string(), rusqlite::types::Type::Text)
            })?;
            
            let created_at_str: String = row.get(7)?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(7, "Invalid datetime".to_string(), rusqlite::types::Type::Text)
                })?
                .with_timezone(&chrono::Utc);
            
            Ok(Habit::from_existing(
                id,
                row.get(1)?, // name
                row.get(2)?, // description
                category,
                frequency,
                row.get(5)?, // target_value
                row.get(6)?, // unit
                created_at,
                row.get(8)?, // is_active
            ))
        });
        
        match result {
            Ok(habit) => Ok(habit),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(StorageError::HabitNotFound {
                    habit_id: habit_id.to_string(),
                })
            },
            Err(e) => Err(StorageError::Query(e)),
        }
    }
    
    /// Update an existing habit
    fn update_habit(&self, habit: &Habit) -> Result<(), StorageError> {
        let category_str = Self::category_to_string(&habit.category);
        let frequency_json = serde_json::to_string(&habit.frequency)?;
        
        let rows_affected = self.conn.execute(
            "UPDATE habits SET 
                name = ?2, 
                description = ?3, 
                category = ?4, 
                frequency_data = ?5,
                target_value = ?6, 
                unit = ?7, 
                is_active = ?8
             WHERE id = ?1",
            params![
                habit.id.to_string(),
                habit.name,
                habit.description,
                category_str,
                frequency_json,
                habit.target_value,
                habit.unit,
                habit.is_active
            ],
        )?;
        
        if rows_affected == 0 {
            return Err(StorageError::HabitNotFound {
                habit_id: habit.id.to_string(),
            });
        }
        
        tracing::debug!("Updated habit: {} ({})", habit.name, habit.id.to_string());
        Ok(())
    }
    
    /// Soft delete a habit (mark as inactive)
    fn delete_habit(&self, habit_id: &HabitId) -> Result<(), StorageError> {
        let rows_affected = self.conn.execute(
            "UPDATE habits SET is_active = 0 WHERE id = ?1",
            params![habit_id.to_string()],
        )?;
        
        if rows_affected == 0 {
            return Err(StorageError::HabitNotFound {
                habit_id: habit_id.to_string(),
            });
        }
        
        tracing::debug!("Soft deleted habit: {}", habit_id.to_string());
        Ok(())
    }
    
    /// List habits with optional filtering
    fn list_habits(
        &self,
        _category: Option<Category>,
        active_only: bool,
    ) -> Result<Vec<Habit>, StorageError> {
        let mut sql = "SELECT id, name, description, category, frequency_data, target_value, unit, created_at, is_active FROM habits".to_string();
        
        if active_only {
            sql.push_str(" WHERE is_active = 1");
        }
        
        sql.push_str(" ORDER BY created_at DESC");
        
        let mut stmt = self.conn.prepare(&sql)?;
        let habit_iter = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let id = HabitId::from_string(&id_str).map_err(|_| {
                rusqlite::Error::InvalidColumnType(0, "Invalid UUID".to_string(), rusqlite::types::Type::Text)
            })?;
            
            let category_str: String = row.get(3)?;
            let category = Self::string_to_category(&category_str).map_err(|_| {
                rusqlite::Error::InvalidColumnType(3, "Invalid category".to_string(), rusqlite::types::Type::Text)
            })?;
            
            let frequency_json: String = row.get(4)?;
            let frequency = serde_json::from_str(&frequency_json).map_err(|_| {
                rusqlite::Error::InvalidColumnType(4, "Invalid frequency".to_string(), rusqlite::types::Type::Text)
            })?;
            
            let created_at_str: String = row.get(7)?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(7, "Invalid datetime".to_string(), rusqlite::types::Type::Text)
                })?
                .with_timezone(&chrono::Utc);
            
            Ok(Habit::from_existing(
                id,
                row.get(1)?, // name
                row.get(2)?, // description
                category,
                frequency,
                row.get(5)?, // target_value
                row.get(6)?, // unit
                created_at,
                row.get(8)?, // is_active
            ))
        })?;
        
        let mut habits = Vec::new();
        for habit in habit_iter {
            habits.push(habit?);
        }
        
        Ok(habits)
    }
    
    /// Create a new habit entry
    fn create_entry(&self, entry: &HabitEntry) -> Result<(), StorageError> {
        self.conn.execute(
            "INSERT INTO habit_entries (
                id, habit_id, logged_at, completed_at, value, intensity, notes
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.id.to_string(),
                entry.habit_id.to_string(),
                entry.logged_at.to_rfc3339(),
                entry.completed_at.to_string(),
                entry.value,
                entry.intensity,
                entry.notes
            ],
        )?;
        
        tracing::debug!("Created habit entry: {} for habit {}", entry.id.to_string(), entry.habit_id.to_string());
        Ok(())
    }
    
    /// Get entries for a specific habit
    fn get_entries_for_habit(
        &self,
        habit_id: &HabitId,
        limit: Option<u32>,
    ) -> Result<Vec<HabitEntry>, StorageError> {
        let sql = if let Some(limit_val) = limit {
            format!("SELECT id, habit_id, logged_at, completed_at, value, intensity, notes 
                     FROM habit_entries WHERE habit_id = ?1 
                     ORDER BY completed_at DESC, logged_at DESC LIMIT {}", limit_val)
        } else {
            "SELECT id, habit_id, logged_at, completed_at, value, intensity, notes 
             FROM habit_entries WHERE habit_id = ?1 
             ORDER BY completed_at DESC, logged_at DESC".to_string()
        };
        
        let mut stmt = self.conn.prepare(&sql)?;
        let entry_iter = stmt.query_map(params![habit_id.to_string()], |row| {
            let entry_id_str: String = row.get(0)?;
            let entry_id = EntryId::from_string(&entry_id_str).map_err(|_| {
                rusqlite::Error::InvalidColumnType(0, "Invalid UUID".to_string(), rusqlite::types::Type::Text)
            })?;
            
            let habit_id_str: String = row.get(1)?;
            let parsed_habit_id = HabitId::from_string(&habit_id_str).map_err(|_| {
                rusqlite::Error::InvalidColumnType(1, "Invalid UUID".to_string(), rusqlite::types::Type::Text)
            })?;
            
            let logged_at_str: String = row.get(2)?;
            let logged_at = chrono::DateTime::parse_from_rfc3339(&logged_at_str)
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(2, "Invalid datetime".to_string(), rusqlite::types::Type::Text)
                })?
                .with_timezone(&chrono::Utc);
            
            let completed_at_str: String = row.get(3)?;
            let completed_at = NaiveDate::parse_from_str(&completed_at_str, "%Y-%m-%d")
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(3, "Invalid date".to_string(), rusqlite::types::Type::Text)
                })?;
            
            Ok(HabitEntry::from_existing(
                entry_id,
                parsed_habit_id,
                logged_at,
                completed_at,
                row.get(4)?, // value
                row.get(5)?, // intensity
                row.get(6)?, // notes
            ))
        })?;
        
        let mut entries = Vec::new();
        for entry in entry_iter {
            entries.push(entry?);
        }
        
        Ok(entries)
    }
    
    /// Get all entries within a date range
    fn get_entries_by_date_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<HabitEntry>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, habit_id, logged_at, completed_at, value, intensity, notes 
             FROM habit_entries 
             WHERE completed_at BETWEEN ?1 AND ?2 
             ORDER BY completed_at DESC, logged_at DESC"
        )?;
        
        let entry_iter = stmt.query_map(
            params![start_date.to_string(), end_date.to_string()], 
            |row| {
                let entry_id_str: String = row.get(0)?;
                let entry_id = EntryId::from_string(&entry_id_str).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(0, "Invalid UUID".to_string(), rusqlite::types::Type::Text)
                })?;
                
                let habit_id_str: String = row.get(1)?;
                let habit_id = HabitId::from_string(&habit_id_str).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(1, "Invalid UUID".to_string(), rusqlite::types::Type::Text)
                })?;
                
                let logged_at_str: String = row.get(2)?;
                let logged_at = chrono::DateTime::parse_from_rfc3339(&logged_at_str)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(2, "Invalid datetime".to_string(), rusqlite::types::Type::Text)
                    })?
                    .with_timezone(&chrono::Utc);
                
                let completed_at_str: String = row.get(3)?;
                let completed_at = NaiveDate::parse_from_str(&completed_at_str, "%Y-%m-%d")
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(3, "Invalid date".to_string(), rusqlite::types::Type::Text)
                    })?;
                
                Ok(HabitEntry::from_existing(
                    entry_id,
                    habit_id,
                    logged_at,
                    completed_at,
                    row.get(4)?, // value
                    row.get(5)?, // intensity
                    row.get(6)?, // notes
                ))
            }
        )?;
        
        let mut entries = Vec::new();
        for entry in entry_iter {
            entries.push(entry?);
        }
        
        Ok(entries)
    }
    
    /// Update or create streak data for a habit
    fn update_streak(&self, streak: &Streak) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT OR REPLACE INTO habit_streaks (
                habit_id, current_streak, longest_streak, last_completed, 
                total_completions, completion_rate, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                streak.habit_id.to_string(),
                streak.current_streak,
                streak.longest_streak,
                streak.last_completed.map(|d| d.to_string()),
                streak.total_completions,
                streak.completion_rate,
                now
            ],
        )?;
        
        tracing::debug!("Updated streak for habit: {}", streak.habit_id.to_string());
        Ok(())
    }
    
    /// Get streak data for a habit
    fn get_streak(&self, habit_id: &HabitId) -> Result<Streak, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT current_streak, longest_streak, last_completed, total_completions, completion_rate 
             FROM habit_streaks WHERE habit_id = ?1"
        )?;
        
        let result = stmt.query_row(params![habit_id.to_string()], |row| {
            let last_completed_str: Option<String> = row.get(2)?;
            let last_completed = last_completed_str
                .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
            
            Ok(Streak {
                habit_id: habit_id.clone(),
                current_streak: row.get(0)?,
                longest_streak: row.get(1)?,
                last_completed,
                total_completions: row.get(3)?,
                completion_rate: row.get(4)?,
            })
        });
        
        match result {
            Ok(streak) => Ok(streak),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // No streak data found, return new empty streak
                Ok(Streak::new(habit_id.clone()))
            },
            Err(e) => Err(StorageError::Query(e)),
        }
    }
    
    /// Get streak data for all habits
    fn get_all_streaks(&self) -> Result<Vec<Streak>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT habit_id, current_streak, longest_streak, last_completed, total_completions, completion_rate 
             FROM habit_streaks"
        )?;
        
        let streak_iter = stmt.query_map([], |row| {
            let habit_id_str: String = row.get(0)?;
            let habit_id = HabitId::from_string(&habit_id_str).map_err(|_| {
                rusqlite::Error::InvalidColumnType(0, "Invalid UUID".to_string(), rusqlite::types::Type::Text)
            })?;
            
            let last_completed_str: Option<String> = row.get(3)?;
            let last_completed = last_completed_str
                .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
            
            Ok(Streak {
                habit_id,
                current_streak: row.get(1)?,
                longest_streak: row.get(2)?,
                last_completed,
                total_completions: row.get(4)?,
                completion_rate: row.get(5)?,
            })
        })?;
        
        let mut streaks = Vec::new();
        for streak in streak_iter {
            streaks.push(streak?);
        }
        
        Ok(streaks)
    }
}