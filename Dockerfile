# Multi-stage build for ForgeGateway
FROM rust:1.70-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Create app directory
WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin forge-gateway

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN addgroup -S forge && adduser -S forge -G forge

# Copy binary from builder stage
COPY --from=builder /app/target/release/forge-gateway /usr/local/bin/

# Create necessary directories
RUN mkdir -p /data && chown forge:forge /data

# Switch to non-root user
USER forge

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV FORGE_CONFIG_PATH=/data/config.toml

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Start the application
CMD ["forge-gateway"]