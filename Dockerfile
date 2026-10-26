# WarGames/JOSHUA Nuclear Risk Assessment System
# Production Dockerfile
# Version: 1.0.0

# Build stage
FROM rust:1.86-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY tests ./tests
COPY benches ./benches
COPY migrations ./migrations

# Build release binary
RUN cargo build --release --locked

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r joshua && useradd -r -g joshua joshua

# Create necessary directories
RUN mkdir -p /app /data /output /logs \
    && chown -R joshua:joshua /app /data /output /logs

# Copy binary from builder
COPY --from=builder /app/target/release/joshua /app/joshua

# Copy configuration template
COPY config.example.toml /app/config.example.toml

# Set ownership
RUN chown -R joshua:joshua /app

# Switch to app user
USER joshua

# Set working directory
WORKDIR /app

# Environment variables
ENV RUST_LOG=info
ENV WARGAMES_CONFIG_DIR=/data/config
ENV WARGAMES_OUTPUT_DIR=/output
ENV WARGAMES_LOG_DIR=/logs

# Expose ports (if needed for future web interface)
# EXPOSE 8080

# Health check
HEALTHCHECK --interval=5m --timeout=30s --start-period=5s --retries=3 \
    CMD ["/app/joshua", "diagnose"] || exit 1

# Default command
ENTRYPOINT ["/app/joshua"]
CMD ["--help"]
