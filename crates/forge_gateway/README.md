# Forgecode Gateway

A gateway service providing HTTP API and WebSocket terminal access for the Forgecode platform.

## Features

- ✅ HTTP REST API with Actix-web
- ✅ WebSocket-based real-time terminal streaming
- ✅ Token-based authentication and validation
- ✅ Integration with Forgecode API layer
- ✅ Command execution endpoints
- ✅ Comprehensive test suite
- ✅ Container deployment ready

## Quick Start

### Using Docker (Recommended)

```bash
# Clone and deploy
git clone <repository>
cd forge_gateway
./deploy.sh
```

The service will be available at `http://localhost:8080`

### Manual Installation

```bash
# Build from source
cargo build --release

# Run the gateway
./target/release/forge_gateway
```

## Configuration

Environment variables:
- `RUST_LOG`: Log level (default: info)
- `BIND_ADDRESS`: Server bind address (default: 0.0.0.0)
- `PORT`: Server port (default: 8080)
- `TOKEN_EXPIRATION_SECONDS`: Token expiration time (default: 3600)

## API Endpoints

- `GET /health` - Health check
- `POST /api/token/validate` - Validate authentication token
- `POST /api/token/renew` - Renew authentication token
- `GET /api/terminal/ws` - WebSocket terminal connection
- `POST /api/command` - Execute command

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## Container Deployment

### Build Image
```bash
docker build -t forgecode/gateway:latest .
```

### Docker Compose
```bash
docker-compose up -d
```

### Kubernetes (Example)
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: forge-gateway
spec:
  replicas: 2
  selector:
    matchLabels:
      app: forge-gateway
  template:
    metadata:
      labels:
        app: forge-gateway
    spec:
      containers:
      - name: gateway
        image: forgecode/gateway:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
```

## Health Checks

The service includes built-in health checks:
- Container health check via Docker
- HTTP health endpoint at `/health`
- Automatic restart on failure

## Monitoring

View logs:
```bash
docker-compose logs -f forge-gateway
```

## License

MIT OR Apache-2.0