# Forgecode Gateway Dockerfile
FROM rust:1.92-slim-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY ./crates/forge_gateway/Cargo.toml ./crates/forge_gateway/

# Build dependencies only (caching optimization)
RUN mkdir -p crates/forge_gateway/src && \
    echo "fn main() {}" > crates/forge_gateway/src/main.rs && \
    cargo build --release --bin forge_gateway && \
    rm -rf target/release/.fingerprint/forge_gateway-*

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin forge_gateway

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser
USER appuser

# Copy binary from builder stage
COPY --from=builder /app/target/release/forge_gateway /usr/local/bin/

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["forge_gateway"]