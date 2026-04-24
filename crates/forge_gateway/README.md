# Forgecode Gateway

A secure, high-performance gateway service for forgecode integration, providing authentication, command execution, file operations, and real-time terminal access via WebSocket.

## Features

- **Authentication System**: JWT-based token authentication with renewal and validation
- **Command Execution**: Secure execution of forgecode commands
- **File Operations**: Read and manage files through the gateway
- **Real-time Terminal**: WebSocket-based terminal interface
- **Static File Serving**: Web UI for administration and monitoring
- **Container-Ready**: Full Docker and Docker Compose support
- **Security**: Rate limiting, CORS, and SSL/TLS support

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust toolchain (for development)

### Using Docker Compose (Recommended)

1. Navigate to the project root:
```bash
cd forgecode-gateway
```

2. Deploy the application:
```bash
chmod +x deploy.sh
./deploy.sh
```

3. Access the application:
   - Web UI: http://localhost:8080/static
   - Health Check: http://localhost:8080/health
   - API Documentation: Available at `/static/docs`

### Manual Build and Run

1. Build the application:
```bash
cargo build --release --bin forgecode-gateway
```

2. Run the application:
```bash
./target/release/forgecode-gateway
```

## Configuration

The gateway can be configured using TOML configuration files. A template is provided in `config/gateway.template.toml`.

### Environment Variables

- `RUST_LOG`: Log level (default: info)
- `BIND_ADDRESS`: Network interface (default: 0.0.0.0)
- `PORT`: Port to listen on (default: 8080)
- `CONFIG_FILE`: Path to configuration file

## API Endpoints

### Authentication

- `POST /auth/token/generate` - Generate a new authentication token
- `POST /auth/token/validate` - Validate an existing token
- `POST /auth/token/renew` - Renew an expiring token

### Forgecode Integration

- `POST /api/command` - Execute a forgecode command
- `POST /api/file/read` - Read a file
- `GET /api/system/info` - Get system information

### WebSocket

- `GET /ws` - WebSocket endpoint for real-time terminal

### Health Check

- `GET /health` - Health check endpoint

## Development

### Building from Source

```bash
# Navigate to the gateway directory
cd crates/forge_gateway

# Build the application
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test -- --test-threads=1

# Run with code coverage
cargo tarpaulin --ignore-tests
```

## Deployment

### Docker

```bash
# Build the Docker image
docker build -t forgecode-gateway .

# Run the container
docker run -p 8080:8080 forgecode-gateway
```

### Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f forgecode-gateway

# Stop services
docker-compose down
```

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Web Browser   │───▶│   Nginx Proxy    │───▶│ Forgecode Gateway│
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                                                │
         │                                                │
         ▼                                                ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Static Files  │    │   Redis Cache    │    │ Forgecode API   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Security

- JWT-based authentication with configurable expiration
- Rate limiting to prevent abuse
- CORS configuration for cross-origin requests
- SSL/TLS termination via Nginx
- Secure token storage and validation
- Input validation and sanitization

## Monitoring

### Health Checks

```bash
curl http://localhost:8080/health
```

### Metrics

Prometheus metrics are available at `/metrics` (if enabled in configuration).

### Logging

Logs are written to stdout and can be configured to write to files. Structured JSON logging is supported.

## Troubleshooting

### Common Issues

1. **Port already in use**: Change the port in the configuration file
2. **Permission denied**: Ensure the application has write access to log directories
3. **Connection refused**: Check if Redis is running and accessible

### Logs

View application logs:

```bash
# Docker Compose
docker-compose logs forgecode-gateway

# Kubernetes
kubectl logs -l app=forgecode-gateway
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions:
- Create an issue in the repository
- Check the documentation in `/static/docs`
- Review the troubleshooting section above