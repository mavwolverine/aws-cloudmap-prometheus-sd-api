# AWS Cloud Map Prometheus Service Discovery API - Dockerfile
#
# Multi-stage build for creating a minimal production image
# Stage 1: Build the Rust application
# Stage 2: Create minimal runtime image

# Build stage
FROM rust:1.87-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml/Cargo.lock changes)
RUN cargo build --release && rm -rf src target/release/deps/aws_cloudmap_prometheus_sd_api*

# Copy source code
COPY src ./src

# Build the actual application
RUN cargo build --release

# Verify the binary was created
RUN ls -la target/release/

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    jq \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create a non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create directory for the application
RUN mkdir -p /app && chown appuser:appuser /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/aws-cloudmap-prometheus-sd-api /usr/local/bin/aws-cloudmap-prometheus-sd-api

# Make binary executable and set ownership
RUN chmod +x /usr/local/bin/aws-cloudmap-prometheus-sd-api

# Copy default configuration (optional)
COPY config.json /app/config.json
RUN chown appuser:appuser /app/config.json

# Copy health check script
COPY health-check.sh /usr/local/bin/health-check.sh
RUN chmod +x /usr/local/bin/health-check.sh

# Set working directory
WORKDIR /app

# Switch to non-root user
USER appuser

# Expose the default port
EXPOSE 3030

# Health check using our custom script
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/health-check.sh

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=3030

# Labels for metadata
LABEL maintainer="AWS Cloud Map Prometheus SD API" \
      description="Prometheus service discovery adapter for AWS Cloud Map" \
      version="1.0" \
      org.opencontainers.image.title="aws-cloudmap-prometheus-sd-api" \
      org.opencontainers.image.description="Prometheus service discovery adapter for AWS Cloud Map" \
      org.opencontainers.image.vendor="AWS" \
      org.opencontainers.image.licenses="Apache-2.0"

# Run the application
ENTRYPOINT ["/usr/local/bin/aws-cloudmap-prometheus-sd-api"]
