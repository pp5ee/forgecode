# Multi-stage Dockerfile for forgecode-gateway
FROM rust:1.75-slim-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin forgecode-gateway

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 forgecode

# Copy binary from builder stage
COPY --from=builder /app/target/release/forgecode-gateway /usr/local/bin/

# Copy static files
COPY --from=builder /app/static /app/static

# Set ownership
RUN chown -R forgecode:forgecode /app

# Switch to non-root user
USER forgecode

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Start the application
CMD ["forgecode-gateway"]