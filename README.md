# Habit Tracker MCP

A Model Context Protocol (MCP) server for intelligent habit tracking and analytics. This server provides tools for creating, tracking, and analyzing habits with advanced insights and streak management.

## Features

- 📝 **Habit Management**: Create, list, and manage habits with flexible frequency patterns
- 📊 **Progress Tracking**: Log entries and track completion rates
- 🔥 **Streak Tracking**: Monitor current and best streaks with intelligent recovery
- 📈 **Analytics**: Generate insights on habit performance and trends
- 🎯 **Status Monitoring**: Quick overview of today's habit status

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

- `create_habit`: Create a new habit with customizable frequency
- `list_habits`: View all habits with their current status
- `log_entry`: Record habit completion
- `get_status`: Check today's habit completion status
- `generate_insights`: Get analytics and performance insights

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
- `habits`: Store habit definitions and metadata
- `entries`: Track individual habit completions
- `streaks`: Monitor streak information

## License

MIT License - see LICENSE file for details.