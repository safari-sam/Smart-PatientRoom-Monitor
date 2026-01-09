# ============================================================================
# Smart Patient Room Monitor - Dockerfile
# ============================================================================
# Multi-stage build for optimized image size
# ============================================================================

# Stage 1: Build the Rust application
FROM rust:slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files first (for dependency caching)
COPY backend/Cargo.toml ./

# Create dummy main.rs to build dependencies (cache layer)
RUN mkdir src && \
    echo "fn main() { println!(\"Dummy build\"); }" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY backend/src ./src

# Build the real application (touch to invalidate cache)
RUN touch src/main.rs && cargo build --release

# ============================================================================
# Stage 2: Runtime image (smaller)
# ============================================================================
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/monitor .

# Copy frontend files
COPY backend/frontend ./frontend

# Set environment variables (defaults)
ENV HOST=0.0.0.0
ENV PORT=8080
ENV DB_HOST=db
ENV DB_PORT=5432
ENV DB_USER=postgres
ENV DB_PASSWORD=postgres
ENV DB_NAME=patient_monitor
ENV MOCK_MODE=true
ENV SOUND_THRESHOLD=150
ENV INACTIVITY_SECONDS=300

# Expose the web server port
EXPOSE 8080

# Run the application
CMD ["./monitor"]
