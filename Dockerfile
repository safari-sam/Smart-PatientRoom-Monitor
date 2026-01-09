# ============================================================================
# Smart Patient Room Monitor - Dockerfile
# ============================================================================

# Stage 1: Build the Rust application
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire backend folder
COPY backend/ ./

# Build the application
RUN cargo build --release

# ============================================================================
# Stage 2: Runtime image (smaller)
# ============================================================================
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    libudev1 \
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
ENV RUST_LOG=info

# Expose the web server port
EXPOSE 8080

# Run the application
CMD ["./monitor"]