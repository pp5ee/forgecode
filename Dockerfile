# Multi-stage Dockerfile for forge_gateway
FROM rust:1.92-slim-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin forge_gateway

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder --chown=appuser:appuser /app/target/release/forge_gateway .

# Copy static files
COPY --chown=appuser:appuser static ./static

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Environment variables
ENV RUST_LOG=info
ENV GATEWAY_PORT=8080
ENV FORGECODE_BASE_URL=http://forgecode:8081

# Start the application
CMD ["./forge_gateway"]