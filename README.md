# Habit Tracker MCP

A Model Context Protocol (MCP) server for intelligent habit tracking and analytics. This server provides tools for creating, tracking, and analyzing habits with advanced insights and streak management.

## Features

- 📝 **Habit Management**: Create, list, and manage habits with flexible frequency patterns
- 📊 **Progress Tracking**: Log entries and track completion rates with detailed metadata
- 🔥 **Advanced Streak Tracking**: Sophisticated streak calculations supporting all frequency types:
  - **Daily**: Consecutive day tracking
  - **Weekdays**: Monday-Friday streak tracking (skips weekends)
  - **Weekends**: Saturday-Sunday streak tracking (skips weekdays)
  - **Weekly**: Consecutive weeks meeting frequency requirements
  - **Interval**: Custom day intervals (e.g., every 3 days)
  - **Custom**: Specific weekday patterns (e.g., Mon/Wed/Fri)
- 📈 **AI-Powered Analytics**: Generate sophisticated insights on habit performance, patterns, and recommendations
- 🎯 **Real-time Status**: Quick overview of current habit status and streaks

## Installation

### Prerequisites

- Rust (1.70 or later)
- Cargo

### Build from source

```bash
git clone <https://github.com/yvie97/HabitTracker-MCP.git>
cd habit-tracker-mcp
cargo build --release
```

## Usage

### As MCP Server

Run the server to provide habit tracking tools to MCP clients:

```bash
cargo run --bin habit-tracker-mcp
```

### Available Tools

- `habit_create`: Create a new habit with customizable frequency patterns (daily, weekdays, weekends, weekly, interval, custom)
- `habit_list`: List habits with detailed analytics including streaks, completion rates, frequency patterns, and sorting options (by streak, completion rate, name, or total completions)
- `habit_log`: Record habit completion with optional intensity, value, and notes
- `habit_status`: Check comprehensive habit status including current/longest streaks and completion rates
- `habit_insights`: Generate AI-powered analytics with performance insights, patterns, and personalized recommendations

### Enhanced Habit Listing

The `habit_list` tool provides rich, detailed information about your habits:

**Features:**
- 📊 **Real-time Analytics**: Current streaks, completion rates, and total completions
- 📅 **Frequency Display**: Human-readable frequency descriptions (Daily, Weekdays, "3 times per week", etc.)
- 🔄 **Smart Sorting**: Sort by streak length, completion rate, total completions, or alphabetically
- 🏷️ **Category Filtering**: Filter habits by category (health, productivity, etc.)
- 📋 **Rich Formatting**: Beautiful emoji-enhanced output with structured data

**Example Output:**
```
📋 Habit Summary (3 habits)

🎯 Morning Exercise (health)
   📅 Frequency: Daily | 🔥 Streak: 7 days | 📊 Rate: 85.7% | ✅ Total: 12

🎯 Reading Practice (productivity)
   📅 Frequency: Weekdays | 🔥 Streak: 3 days | 📊 Rate: 75.0% | ✅ Total: 9

📊 Overall Stats
- Active habits: 2
- Average completion rate: 80.4%
```

## Configuration

The server stores data in a SQLite database located at:
- macOS: `~/Library/Application Support/habit-tracker-mcp/habits.db`
- Linux: `~/.local/share/habit-tracker-mcp/habits.db`
- Windows: `%APPDATA%\habit-tracker-mcp\habits.db`

## Development

### Running Tests

```bash
cargo test
```

### Database Schema

The application uses SQLite with the following main tables:
- `habits`: Store habit definitions, categories, and frequency patterns
- `entries`: Track individual habit completions with timestamps, intensity, and notes
- Advanced streak calculations are computed dynamically from entry data

### Testing

Run the comprehensive test suite including streak calculation tests:

```bash
# Run Rust unit and integration tests
cargo test

# Run MCP protocol tests
python3 tests/test_mcp.py
```

## License

MIT License - see LICENSE file for details.