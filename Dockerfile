# Build stage
FROM rust:1.75-slim-bullseye AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false habit-tracker

# Create data directory
RUN mkdir -p /app/data && chown habit-tracker:habit-tracker /app/data

# Copy the binary from builder stage
COPY --from=builder /app/target/release/habit-tracker-mcp /usr/local/bin/habit-tracker-mcp

# Set correct permissions
RUN chmod +x /usr/local/bin/habit-tracker-mcp

# Switch to non-root user
USER habit-tracker

# Set working directory
WORKDIR /app

# Expose port (if needed for HTTP interface)
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD pgrep habit-tracker-mcp || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV HABIT_TRACKER_DATA_DIR=/app/data

# Run the application
CMD ["habit-tracker-mcp"]