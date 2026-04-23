# ForgeGateway Deployment Guide

## Quick Start

### Using Docker Compose (Recommended)

1. Copy the example configuration:
   ```bash
   cp config.toml.example config.toml
   ```

2. Edit `config.toml` with your ForgeCode API credentials:
   ```toml
   [forge_api]
   base_url = "https://api.forgecode.com"
   auth_token = "your-actual-token"
   ```

3. Start the service:
   ```bash
   docker-compose up -d
   ```

4. Verify the service is running:
   ```bash
   curl http://localhost:8080/health
   ```

### Manual Build & Run

1. Build the application:
   ```bash
   cargo build --release
   ```

2. Create a configuration file:
   ```bash
   cp config.toml.example config.toml
   # Edit config.toml with your settings
   ```

3. Run the application:
   ```bash
   FORGE_CONFIG_PATH=./config.toml ./target/release/forge-gateway
   ```

## Environment Variables

- `FORGE_CONFIG_PATH`: Path to configuration file (default: `./config.toml`)
- `RUST_LOG`: Logging level (default: `info`)

## Health Check

The service provides a health endpoint at `/health` that returns:
- `200 OK` when service is healthy
- `503 Service Unavailable` when service is unhealthy

## Monitoring

The service includes:
- Structured JSON logging
- Health check endpoint
- Rate limiting (1000 requests/minute)
- CORS support for web applications

## Production Considerations

1. **Security**: Set proper CORS origins and rate limits
2. **Authentication**: Use environment variables for API tokens
3. **Monitoring**: Set up log aggregation and alerting
4. **Scaling**: Use load balancer for multiple instances
5. **Backup**: Regularly backup configuration and data volumes