# Forge Gateway Deployment Guide

## Overview

The Forge Gateway is a Rust-based service that provides authentication, WebSocket communication, and HTTP API endpoints for interacting with the forgecode service. It acts as a secure gateway between clients and the code execution service.

## Prerequisites

- Rust 1.70+ and Cargo
- Forgecode service running (optional - gateway can operate in mock mode)
- Environment variables configured

## Quick Start

### 1. Clone and Build

```bash
git clone <repository>
cd forge_gateway
cargo build --release
```

### 2. Configure Environment

Create a `.env` file:

```bash
# Gateway configuration
GATEWAY_PORT=8080

# Forgecode service URL (optional)
FORGECODE_BASE_URL=http://localhost:8081

# Token expiration (optional, default: 3600 seconds)
TOKEN_EXPIRATION_SECONDS=3600
```

### 3. Run the Gateway

```bash
# Using cargo (development)
cargo run

# Using the built binary
./target/release/forge_gateway
```

## Configuration

### Environment Variables

- `GATEWAY_PORT`: Port to run the gateway on (default: 8080)
- `FORGECODE_BASE_URL`: URL of the forgecode service (default: http://localhost:8081)
- `TOKEN_EXPIRATION_SECONDS`: Token expiration time in seconds (default: 3600)

### Runtime Configuration

The gateway can also be configured programmatically:

```rust
use forge_gateway::GatewayConfig;

let config = GatewayConfig {
    port: 8080,
    forgecode_base_url: "http://forgecode:8081".to_string(),
    token_expiration_seconds: 7200,
};

forge_gateway::start_gateway(config).await?;
```

## API Endpoints

### HTTP API

#### Execute Code
```http
POST /api/execute
Content-Type: application/json

{
    "code": "print('hello world')",
    "language": "python",
    "session_id": "optional-session-id"
}
```

#### Token Management

- **Generate Token**: `POST /api/auth/generate`
- **Validate Token**: `POST /api/auth/validate`
- **Renew Token**: `POST /api/auth/renew`
- **Revoke Token**: `POST /api/auth/revoke`

### WebSocket API

Connect to `ws://localhost:8080/ws` and send JSON messages:

#### Authentication
```json
{
    "action": "authenticate",
    "token": "user-id"
}
```

#### Code Execution
```json
{
    "action": "execute",
    "code": "print('hello')",
    "language": "python",
    "session_id": "session-123",
    "token": "auth-token"
}
```

#### Token Validation
```json
{
    "action": "validate",
    "token": "auth-token"
}
```

## Authentication System

### Token-Based Authentication

The gateway uses UUID-based tokens with the following features:

- **Expiration**: Configurable token lifetime
- **Permissions**: Fine-grained permission system
- **Renewal**: Token renewal without re-authentication
- **Revocation**: Immediate token invalidation

### Permission Levels

- `execute`: Execute code through the gateway
- `read`: Access execution results and logs
- `admin`: Administrative operations

## Integration with Forgecode Service

### Service Discovery

The gateway automatically discovers and connects to the forgecode service. If the service is unavailable, the gateway falls back to mock execution mode.

### Health Checks

```bash
# Check gateway health
curl http://localhost:8080/api/health

# Check forgecode service health
curl http://forgecode:8081/health
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/forge_gateway /usr/local/bin/
EXPOSE 8080
CMD ["forge_gateway"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  forge_gateway:
    build: .
    ports:
      - "8080:8080"
    environment:
      - GATEWAY_PORT=8080
      - FORGECODE_BASE_URL=http://forgecode:8081
    depends_on:
      - forgecode

  forgecode:
    image: forgecode:latest
    ports:
      - "8081:8081"
```

## Monitoring and Logging

### Logging

The gateway uses `env_logger` for structured logging. Configure log level:

```bash
RUST_LOG=info forge_gateway
RUST_LOG=debug forge_gateway  # For detailed debugging
```

### Metrics

Key metrics to monitor:
- Request rate and latency
- WebSocket connections
- Token operations
- Forgecode service availability

## Security Considerations

### Network Security

- Use HTTPS/TLS in production
- Configure firewall rules
- Implement rate limiting
- Use secure WebSocket connections (WSS)

### Authentication Security

- Use strong token generation
- Implement token rotation
- Monitor for suspicious activity
- Set appropriate token expiration times

### Code Execution Security

- Validate all input parameters
- Implement code size limits
- Use sandboxed execution environments
- Monitor execution patterns

## Troubleshooting

### Common Issues

1. **Forgecode service unavailable**: Gateway falls back to mock mode
2. **Token validation failures**: Check token expiration and permissions
3. **WebSocket connection issues**: Verify CORS and network configuration
4. **High memory usage**: Monitor connection count and code execution patterns

### Debug Mode

Run in debug mode for detailed logs:

```bash
RUST_LOG=debug cargo run
```

## Performance Tuning

### Memory Optimization

- Configure connection limits
- Implement connection pooling
- Monitor memory usage patterns

### Network Optimization

- Use connection keep-alive
- Implement request batching
- Configure appropriate timeouts

## Scaling

### Horizontal Scaling

- Use load balancer with sticky sessions
- Implement shared token storage (Redis)
- Configure service discovery

### Vertical Scaling

- Increase memory allocation
- Optimize database connections
- Tune thread pool sizes

## Backup and Recovery

### Data Backup

- Backup token database (if persistent)
- Save configuration files
- Export logs and metrics

### Disaster Recovery

- Implement automated failover
- Maintain backup instances
- Test recovery procedures regularly

## Support and Maintenance

### Monitoring

- Set up health checks
- Monitor error rates
- Track performance metrics

### Updates

- Regular security updates
- Dependency updates
- Feature enhancements

## Contributing

See the project's CONTRIBUTING.md for development guidelines and contribution process.