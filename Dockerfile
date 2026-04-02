# Multi-stage Dockerfile for Hoang-Harness (Claw-Harness)
# Build stage
FROM rust:slim AS builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Set optimization flags
ENV RUSTFLAGS="-C target-cpu=native"

# Build release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/claw /usr/local/bin/claw

# Set working directory
WORKDIR /workspace

# Default entrypoint
ENTRYPOINT ["claw"]
