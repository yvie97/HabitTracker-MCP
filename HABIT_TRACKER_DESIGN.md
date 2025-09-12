# Habit Tracker MCP - Detailed Design Document

## Table of Contents
1. [Project Overview](#project-overview)
2. [Goals and Objectives](#goals-and-objectives)
3. [User Stories](#user-stories)
4. [Domain Model](#domain-model)
5. [MCP Tools Specification](#mcp-tools-specification)
6. [Analytics and Insights](#analytics-and-insights)
7. [Data Storage](#data-storage)
8. [Technical Architecture](#technical-architecture)
9. [API Examples](#api-examples)
10. [Implementation Roadmap](#implementation-roadmap)

---

## Project Overview

The **Habit Tracker MCP** is a Model Context Protocol server that enables AI assistants to help users build and maintain positive habits through intelligent tracking, analytics, and motivation.

### What is Habit Tracking?
Habit tracking is the practice of monitoring behaviors you want to develop into automatic routines. Research shows that tracking increases success rates by:
- Making progress visible
- Creating accountability
- Identifying patterns and triggers
- Building motivation through streaks

### Why an MCP Server?
By implementing this as an MCP server, we enable AI assistants like Claude to:
- Naturally integrate habit tracking into conversations
- Provide contextual motivation and insights
- Generate personalized recommendations
- Make habit tracking feel conversational rather than administrative

---

## Goals and Objectives

### Primary Goals
1. **Simplify Habit Formation**: Make it easy to start tracking any habit
2. **Provide Insights**: Generate meaningful analytics about habit patterns
3. **Maintain Motivation**: Use streaks and achievements to encourage consistency
4. **Enable AI Integration**: Allow seamless interaction through conversational AI

### Success Metrics
- Users can create and log habits within 30 seconds
- System provides actionable insights after 1 week of data
- Streak calculations motivate continued engagement
- AI can naturally discuss habits and progress

---

## User Stories

### Core User Stories

**Story 1: Getting Started**
> As a user wanting to build better habits, I want to easily define what I want to track, so that I can start building consistency.

**Story 2: Daily Logging**
> As someone working on habits, I want to quickly log my daily activities with optional notes, so that I can track progress without friction.

**Story 3: Progress Tracking**
> As a habit tracker, I want to see my current streaks and recent activity, so that I can stay motivated and see my progress.

**Story 4: Pattern Recognition**
> As someone improving my life, I want to understand which habits work well together, so that I can optimize my routine.

**Story 5: Motivation and Recovery**
> As someone who sometimes misses days, I want encouragement and strategies to get back on track, so that I don't give up entirely.

### Advanced User Stories

**Story 6: Habit Evolution**
> As my routine matures, I want to modify habit frequency or intensity, so that my tracking evolves with my lifestyle.

**Story 7: Category Analysis**
> As someone tracking multiple life areas, I want to see how different categories (health, productivity, social) are performing relative to each other.

---

## Domain Model

### Core Entities

#### 1. Habit
A habit represents something the user wants to do regularly.

```rust
pub struct Habit {
    pub id: HabitId,           // Unique identifier
    pub name: HabitName,       // "Morning Run", "Read for 30min"
    pub description: Option<String>, // Optional detailed description
    pub category: Category,     // health, productivity, social, etc.
    pub frequency: Frequency,   // How often it should be done
    pub target_value: Option<u32>, // Optional numeric target (minutes, reps)
    pub unit: Option<String>,   // "minutes", "pages", "reps"
    pub created_at: DateTime<Utc>,
    pub is_active: bool,       // Can be paused/disabled
}
```

#### 2. HabitEntry
A record of completing a habit on a specific day.

```rust
pub struct HabitEntry {
    pub id: EntryId,
    pub habit_id: HabitId,
    pub logged_at: DateTime<Utc>,    // When the entry was created
    pub completed_at: Date<Utc>,     // Which day this was completed
    pub value: Option<u32>,          // Actual amount (if habit has target)
    pub intensity: Option<u8>,       // Subjective rating 1-10
    pub notes: Option<String>,       // User's notes about this completion
}
```

#### 3. Streak
Calculated streak information for a habit.

```rust
pub struct Streak {
    pub habit_id: HabitId,
    pub current_streak: u32,         // Current consecutive days
    pub longest_streak: u32,         // Best streak ever achieved
    pub last_completed: Option<Date<Utc>>,
    pub total_completions: u32,      // All-time completion count
    pub completion_rate: f64,        // Percentage since habit creation
}
```

### Supporting Types

#### Frequency
How often a habit should be performed.

```rust
pub enum Frequency {
    Daily,                    // Every day
    Weekly(u8),              // X times per week (1-7)
    Weekdays,                // Monday through Friday only  
    Weekends,                // Saturday and Sunday only
    Custom(Vec<Weekday>),    // Specific days of week
    Interval(u32),           // Every N days
}
```

#### Category
Broad areas of life for organizing habits.

```rust
pub enum Category {
    Health,        // Exercise, diet, sleep
    Productivity,  // Work, learning, organization
    Social,        // Relationships, communication
    Creative,      // Art, writing, music
    Mindfulness,   // Meditation, reflection, gratitude
    Financial,     // Budgeting, investing, spending tracking
    Household,     // Cleaning, maintenance, organization
    Personal,      // Hobbies, self-care, personal growth
    Custom(String), // User-defined category
}
```

### Validation Rules

#### Habit Validation
- `name`: 1-100 characters, non-empty
- `frequency`: Must be valid enum value
- `target_value`: If present, must be > 0
- `unit`: If present, 1-20 characters

#### HabitEntry Validation
- `completed_at`: Cannot be in future
- `value`: If habit has target, value should be reasonable (0-10x target)
- `intensity`: Must be 1-10 if present
- `notes`: Max 500 characters

---

## MCP Tools Specification

### Tool 1: `habit_create`

**Purpose**: Create a new habit to track.

**Parameters**:
```json
{
  "name": "string (required)",
  "description": "string (optional)", 
  "category": "enum (required)",
  "frequency": "enum (required)",
  "target_value": "number (optional)",
  "unit": "string (optional)"
}
```

**Example Request**:
```json
{
  "name": "Morning Run",
  "description": "30-minute jog around the neighborhood",
  "category": "health", 
  "frequency": "daily",
  "target_value": 30,
  "unit": "minutes"
}
```

**Response**:
```json
{
  "success": true,
  "habit_id": "hab_123",
  "message": "✅ Created habit 'Morning Run' - ready to start your streak!"
}
```

### Tool 2: `habit_log`

**Purpose**: Log completion of a habit for a specific day.

**Parameters**:
```json
{
  "habit_id": "string (required)",
  "completed_at": "date (optional, defaults to today)",
  "value": "number (optional)", 
  "intensity": "number 1-10 (optional)",
  "notes": "string (optional)"
}
```

**Example Request**:
```json
{
  "habit_id": "hab_123",
  "value": 25,
  "intensity": 7,
  "notes": "Felt great! Ran the whole way without stopping."
}
```

**Response**:
```json
{
  "success": true,
  "message": "🔥 Logged 'Morning Run'! Current streak: 5 days",
  "current_streak": 5,
  "streak_milestone": "halfway to your longest streak of 10 days!"
}
```

### Tool 3: `habit_status`

**Purpose**: Get current status and streaks for all or specific habits.

**Parameters**:
```json
{
  "habit_id": "string (optional - if omitted, returns all)",
  "include_recent": "boolean (optional, default true)"
}
```

**Response**:
```json
{
  "habits": [
    {
      "habit_id": "hab_123",
      "name": "Morning Run", 
      "current_streak": 5,
      "longest_streak": 10,
      "completion_rate": 0.85,
      "last_completed": "2024-01-15",
      "status": "on_track"
    }
  ],
  "recent_entries": [
    {
      "habit_name": "Morning Run",
      "completed_at": "2024-01-15", 
      "value": 25,
      "intensity": 7
    }
  ]
}
```

### Tool 4: `habit_insights`

**Purpose**: Generate analytical insights about habit patterns.

**Parameters**:
```json
{
  "timeframe": "enum (week|month|all_time)",
  "insight_type": "enum (streaks|patterns|recommendations|all)"
}
```

**Response**:
```json
{
  "insights": [
    {
      "type": "streak_analysis", 
      "message": "Your longest streaks happen when you log habits in the morning",
      "data": {"morning_completion_rate": 0.92, "evening_completion_rate": 0.68}
    },
    {
      "type": "correlation",
      "message": "When you complete 'Morning Run', you're 3x more likely to complete 'Healthy Lunch'",
      "data": {"correlation_strength": 0.78}
    },
    {
      "type": "recommendation",
      "message": "Consider adding a rest day - your intensity drops on day 6+ of streaks",
      "data": {"avg_intensity_days_1_5": 8.2, "avg_intensity_days_6_plus": 6.1}
    }
  ]
}
```

### Tool 5: `habit_list`

**Purpose**: List all habits with summary statistics.

**Parameters**:
```json
{
  "category": "enum (optional)",
  "active_only": "boolean (optional, default true)",
  "sort_by": "enum (name|streak|created_at|completion_rate)"
}
```

**Response**:
```json
{
  "habits": [
    {
      "habit_id": "hab_123",
      "name": "Morning Run",
      "category": "health", 
      "frequency": "daily",
      "current_streak": 5,
      "completion_rate": 0.85,
      "total_completions": 23,
      "is_active": true
    }
  ],
  "summary": {
    "total_habits": 1,
    "active_habits": 1, 
    "avg_completion_rate": 0.85
  }
}
```

### Tool 6: `habit_update`

**Purpose**: Modify an existing habit's properties.

**Parameters**:
```json
{
  "habit_id": "string (required)",
  "name": "string (optional)",
  "description": "string (optional)",
  "frequency": "enum (optional)", 
  "target_value": "number (optional)",
  "unit": "string (optional)",
  "is_active": "boolean (optional)"
}
```

---

## Analytics and Insights

### Streak Calculation Logic

#### Basic Streak Rules
1. **Daily Habits**: Streak breaks if missed for 24+ hours
2. **Weekly Habits**: Streak continues as long as weekly target is met
3. **Custom Frequency**: Streak based on scheduled days only

#### Grace Period System
- **Sick Days**: User can mark days as "sick" to maintain streak
- **Travel Mode**: Reduced frequency during travel periods
- **Flexible Streaks**: Allow 1 miss per 7 days for daily habits

### Pattern Recognition

#### Correlation Analysis
- **Habit Chains**: Identify habits that often happen together
- **Time Patterns**: When during day/week are habits most successful
- **Environmental Factors**: Day of week, season, etc.

#### Insight Types

1. **Performance Insights**
   - "You're most consistent on Mondays" 
   - "Your completion rate improves when you log in the morning"
   - "Habits with targets have 23% higher completion rates"

2. **Behavioral Insights**
   - "When you complete A, you're X% more likely to complete B"
   - "Your intensity scores are highest on weekends"
   - "You tend to skip habits after 3+ consecutive days"

3. **Motivational Insights**  
   - "You're 2 days away from your longest streak!"
   - "This week's completion rate is 15% above your average"
   - "You've completed habits 23 days this month - personal record!"

### Recommendation Engine

#### Smart Suggestions
- **Habit Stacking**: Suggest pairing new habits with established ones
- **Optimal Timing**: Recommend best times based on historical data
- **Frequency Adjustment**: Suggest frequency changes based on performance

#### Recovery Strategies
- **Streak Recovery**: "Start with just 5 minutes to rebuild momentum"
- **Pattern Breaking**: "Try doing this habit at a different time"
- **Simplification**: "Consider reducing the target from 30 to 15 minutes"

---

## Data Storage

### Storage Requirements

#### Data Persistence
- **Local Storage**: SQLite database for local-first approach
- **Cross-Session**: Data survives server restarts
- **Backup Ready**: Easy to export/import data

#### Performance Considerations
- **Read Heavy**: Most operations are queries (status, insights)
- **Small Dataset**: Even power users unlikely to exceed 10MB
- **Simple Queries**: Mostly single-table or simple joins

### Database Schema

#### Tables

**habits**
```sql
CREATE TABLE habits (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    frequency_type TEXT NOT NULL,  -- 'daily', 'weekly', etc.
    frequency_data TEXT,           -- JSON for complex frequencies
    target_value INTEGER,
    unit TEXT,
    created_at TEXT NOT NULL,
    is_active BOOLEAN DEFAULT TRUE
);
```

**habit_entries**
```sql
CREATE TABLE habit_entries (
    id TEXT PRIMARY KEY,
    habit_id TEXT NOT NULL,
    logged_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,     -- Date only (YYYY-MM-DD)
    value INTEGER,
    intensity INTEGER,
    notes TEXT,
    FOREIGN KEY (habit_id) REFERENCES habits (id)
);
```

**habit_streaks** (calculated/cached)
```sql
CREATE TABLE habit_streaks (
    habit_id TEXT PRIMARY KEY,
    current_streak INTEGER NOT NULL DEFAULT 0,
    longest_streak INTEGER NOT NULL DEFAULT 0,
    last_completed TEXT,
    total_completions INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (habit_id) REFERENCES habits (id)
);
```

#### Indexes
```sql
CREATE INDEX idx_habit_entries_habit_completed ON habit_entries (habit_id, completed_at);
CREATE INDEX idx_habit_entries_completed_at ON habit_entries (completed_at);
CREATE INDEX idx_habits_category ON habits (category);
```

---

## Technical Architecture

### Project Structure
```
habit-tracker-mcp/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # Server entry point
│   ├── lib.rs               # Public API exports
│   ├── server.rs            # MCP server implementation
│   ├── domain/
│   │   ├── mod.rs           # Domain types and validation
│   │   ├── habit.rs         # Habit entity and validation
│   │   ├── entry.rs         # HabitEntry entity
│   │   └── streak.rs        # Streak calculation logic
│   ├── storage/
│   │   ├── mod.rs           # Storage abstraction
│   │   ├── sqlite.rs        # SQLite implementation
│   │   └── migrations.rs    # Database schema management
│   ├── analytics/
│   │   ├── mod.rs           # Analytics engine
│   │   ├── streaks.rs       # Streak calculation
│   │   ├── insights.rs      # Pattern recognition
│   │   └── recommendations.rs # Smart suggestions
│   └── tools/
│       ├── mod.rs           # MCP tool implementations
│       ├── create.rs        # habit_create tool
│       ├── log.rs           # habit_log tool
│       ├── status.rs        # habit_status tool
│       ├── insights.rs      # habit_insights tool
│       ├── list.rs          # habit_list tool
│       └── update.rs        # habit_update tool
└── tests/
    ├── integration/         # End-to-end tests
    └── unit/               # Unit tests
```

### Key Dependencies

```toml
[dependencies]
# MCP Protocol
rmcp = { version = "0.3", features = ["server", "macros", "transport-io"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Database
rusqlite = { version = "0.30", features = ["bundled", "chrono"] }

# Validation
nutype = { version = "0.6", features = ["serde"] }

# Error Handling  
snafu = "0.8"

# Async Runtime
tokio = { version = "1", features = ["macros", "rt-multi-thread", "io-std"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
bon = "3"
```

### Error Handling Strategy

#### Error Types
```rust
#[derive(Debug, Snafu)]
pub enum HabitError {
    #[snafu(display("Habit not found: {habit_id}"))]
    HabitNotFound { habit_id: String },
    
    #[snafu(display("Invalid habit data: {message}"))]
    ValidationError { message: String },
    
    #[snafu(display("Database error: {source}"))]
    DatabaseError { source: rusqlite::Error },
    
    #[snafu(display("Entry already exists for {habit_id} on {date}"))]
    DuplicateEntry { habit_id: String, date: String },
}
```

#### Error Mapping for MCP
```rust
impl From<HabitError> for rmcp::ErrorData {
    fn from(err: HabitError) -> Self {
        match err {
            HabitError::HabitNotFound { .. } => {
                rmcp::ErrorData::invalid_params(&err.to_string())
            }
            HabitError::ValidationError { .. } => {
                rmcp::ErrorData::invalid_params(&err.to_string()) 
            }
            HabitError::DatabaseError { .. } => {
                rmcp::ErrorData::internal_error(&err.to_string())
            }
            HabitError::DuplicateEntry { .. } => {
                rmcp::ErrorData::invalid_params(&err.to_string())
            }
        }
    }
}
```

---

## API Examples

### Example 1: Getting Started with Habits

**Step 1: Create first habit**
```json
// AI Assistant: "Let's set up habit tracking! What would you like to work on?"
// User: "I want to read more books"

{
  "tool": "habit_create",
  "params": {
    "name": "Daily Reading",
    "description": "Read fiction or non-fiction for personal growth",
    "category": "personal",
    "frequency": "daily", 
    "target_value": 30,
    "unit": "minutes"
  }
}

// Response
{
  "success": true,
  "habit_id": "hab_reading_001",
  "message": "📚 Created habit 'Daily Reading'! Start your streak today by reading for 30 minutes."
}
```

**Step 2: Log first completion**
```json
{
  "tool": "habit_log", 
  "params": {
    "habit_id": "hab_reading_001",
    "value": 35,
    "intensity": 8,
    "notes": "Read 'Atomic Habits' chapter 3. Really engaging!"
  }
}

// Response
{
  "success": true,
  "message": "🔥 Logged 'Daily Reading'! Current streak: 1 day. Great start!",
  "current_streak": 1,
  "streak_milestone": "Keep it up to reach your first week!"
}
```

### Example 2: Weekly Progress Check

**Check status after a week**
```json
{
  "tool": "habit_status",
  "params": {
    "include_recent": true
  }
}

// Response
{
  "habits": [
    {
      "habit_id": "hab_reading_001", 
      "name": "Daily Reading",
      "current_streak": 5,
      "longest_streak": 5, 
      "completion_rate": 0.71,
      "last_completed": "2024-01-19",
      "status": "strong_momentum"
    }
  ],
  "recent_entries": [
    {
      "habit_name": "Daily Reading",
      "completed_at": "2024-01-19",
      "value": 25,
      "intensity": 7,
      "notes": "Shorter session but stayed consistent"
    },
    {
      "habit_name": "Daily Reading", 
      "completed_at": "2024-01-18",
      "value": 40,
      "intensity": 9,
      "notes": "Couldn't put the book down!"
    }
  ],
  "summary": "📖 You've read 5 out of 7 days this week! Your average session is 32 minutes."
}
```

### Example 3: Getting Insights

**Request insights after 30 days**
```json
{
  "tool": "habit_insights",
  "params": {
    "timeframe": "month",
    "insight_type": "all"
  }
}

// Response  
{
  "insights": [
    {
      "type": "streak_analysis",
      "title": "Peak Performance Times",
      "message": "You're most consistent when logging habits in the evening (87% completion rate vs 62% in morning)",
      "data": {
        "evening_completions": 18,
        "morning_completions": 8,
        "evening_rate": 0.87,
        "morning_rate": 0.62
      }
    },
    {
      "type": "intensity_pattern",
      "title": "Quality Trends", 
      "message": "Your reading intensity peaks mid-week (Tuesday-Thursday average: 8.3/10)",
      "data": {
        "weekday_avg_intensity": 8.3,
        "weekend_avg_intensity": 6.7
      }
    },
    {
      "type": "recommendation",
      "title": "Optimization Suggestion",
      "message": "Consider scheduling reading sessions for Tuesday-Thursday evenings to maximize both consistency and enjoyment",
      "confidence": 0.82
    }
  ],
  "summary": "🎯 You've maintained a 73% completion rate this month with an average intensity of 7.4/10. Your longest streak was 8 days!"
}
```

---

## Implementation Roadmap

### Phase 1: Core Foundation (Week 1-2)
- [ ] Set up project structure and dependencies
- [ ] Implement domain types (Habit, HabitEntry, basic validation)  
- [ ] Create SQLite storage layer with migrations
- [ ] Build basic MCP server with `habit_create` and `habit_log` tools
- [ ] Add unit tests for domain logic

### Phase 2: Essential Features (Week 3-4)
- [ ] Implement `habit_status` and `habit_list` tools
- [ ] Add basic streak calculation logic
- [ ] Create `habit_update` tool for modifying habits
- [ ] Add integration tests for MCP tools
- [ ] Implement graceful error handling

### Phase 3: Analytics Engine (Week 5-6)  
- [ ] Build streak calculation with grace periods
- [ ] Implement `habit_insights` tool
- [ ] Add pattern recognition (time-of-day, correlation analysis)
- [ ] Create recommendation engine basics
- [ ] Add comprehensive test coverage

### Phase 4: Polish and Advanced Features (Week 7-8)
- [ ] Enhance insight generation with more patterns
- [ ] Add habit categories and filtering
- [ ] Implement data export/import functionality  
- [ ] Create comprehensive documentation
- [ ] Performance optimization and edge case handling

### Phase 5: Deployment and Documentation (Week 9-10)
- [ ] Create Docker container for easy deployment
- [ ] Write comprehensive README with examples
- [ ] Add MCP server configuration examples
- [ ] Create demo videos or interactive examples
- [ ] Publish to container registry

### Development Guidelines

#### Testing Strategy
- **Unit Tests**: All domain logic, validation, streak calculations
- **Integration Tests**: MCP tool interactions, database operations  
- **Property Tests**: Streak calculation edge cases, data consistency

#### Code Quality Standards
- **Documentation**: All public APIs documented with examples
- **Error Messages**: User-friendly error messages for all failure cases
- **Performance**: Sub-100ms response times for all operations
- **Memory**: Efficient queries, minimal data loading

#### Success Criteria
- [ ] Can create and track habits within 30 seconds
- [ ] Provides meaningful insights after 1 week of data
- [ ] Streak calculations are accurate and motivating
- [ ] AI can naturally discuss habits through MCP interface
- [ ] System is reliable and handles edge cases gracefully

---

## Conclusion

This design document provides a comprehensive blueprint for building an innovative and useful Habit Tracker MCP server. The system balances simplicity for quick adoption with sophisticated analytics that provide real value over time.

The modular architecture allows for incremental development, starting with core habit tracking and evolving toward intelligent insights and recommendations. The MCP integration makes habit tracking feel natural within AI conversations rather than a separate administrative task.

Key innovations include flexible frequency patterns, correlation analysis between habits, intelligent streak recovery, and AI-powered insights that help users optimize their routines for maximum success.