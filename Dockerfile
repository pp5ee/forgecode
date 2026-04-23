# ForgeCode Gateway Docker Image
# Multi-stage build for optimized production image

# Stage 1: Build stage
FROM rust:1.92-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy source code
COPY . .

# Build the gateway in release mode
RUN cargo build --release --package forge_gateway

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 forgecode

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/forge_gateway /app/forge_gateway

# Copy static files
COPY --from=builder /app/crates/forge_gateway/static /app/static

# Set ownership
RUN chown -R forgecode:forgecode /app

# Switch to non-root user
USER forgecode

# Expose the gateway port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# Environment variables
ENV RUST_LOG=info
ENV FORGE_GATEWAY_PORT=8080
ENV FORGE_GATEWAY_BIND=0.0.0.0

# Start the gateway
CMD ["/app/forge_gateway"]