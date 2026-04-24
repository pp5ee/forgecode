#!/bin/bash

# Forgecode Gateway Deployment Script
# This script builds and deploys the forgecode gateway with container orchestration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="forgecode-gateway"
DOCKER_REGISTRY="" # Set to your registry if using one
ENVIRONMENT="${1:-development}"

# Function to print colored output
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi

    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi

    # Check if we're in the right directory
    if [[ ! -f "Cargo.toml" ]]; then
        log_error "This script must be run from the project root directory."
        exit 1
    fi

    log_success "Prerequisites check passed"
}

# Function to build the application
build_application() {
    log_info "Building the application..."

    # Build the Rust application
    log_info "Building Rust binary..."
    cargo build --release --bin forgecode-gateway

    if [[ $? -ne 0 ]]; then
        log_error "Failed to build the Rust application"
        exit 1
    fi

    log_success "Application built successfully"
}

# Function to build Docker images
build_docker_images() {
    log_info "Building Docker images..."

    # Build the main application image
    docker build -t "${PROJECT_NAME}:latest" .

    if [[ $? -ne 0 ]]; then
        log_error "Failed to build Docker image"
        exit 1
    fi

    # Tag for registry if specified
    if [[ -n "$DOCKER_REGISTRY" ]]; then
        docker tag "${PROJECT_NAME}:latest" "${DOCKER_REGISTRY}/${PROJECT_NAME}:latest"
        docker tag "${PROJECT_NAME}:latest" "${DOCKER_REGISTRY}/${PROJECT_NAME}:${ENVIRONMENT}"
    fi

    log_success "Docker images built successfully"
}

# Function to create configuration directories
setup_configuration() {
    log_info "Setting up configuration..."

    # Create necessary directories
    mkdir -p config logs data nginx/ssl

    # Generate self-signed SSL certificates for development
    if [[ ! -f "nginx/ssl/server.crt" ]] && [[ "$ENVIRONMENT" == "development" ]]; then
        log_info "Generating self-signed SSL certificates..."
        openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
            -keyout nginx/ssl/server.key \
            -out nginx/ssl/server.crt \
            -subj "/C=US/ST=State/L=City/O=Organization/OU=Unit/CN=localhost" 2>/dev/null
    fi

    # Create environment-specific configuration
    if [[ ! -f "config/gateway.${ENVIRONMENT}.toml" ]]; then
        cat > "config/gateway.${ENVIRONMENT}.toml" << EOF
[server]
bind_address = "0.0.0.0"
port = 8080

[auth]
token_expiry_hours = 24
secret_key = "change-this-in-production"

[logging]
level = "info"
format = "json"

[forgecode]
api_url = "http://forgecode-api:3000"
api_key = ""
EOF
        log_warning "Created default configuration file. Please review and update config/gateway.${ENVIRONMENT}.toml"
    fi

    log_success "Configuration setup completed"
}

# Function to deploy the application
deploy_application() {
    log_info "Deploying application..."

    # Stop existing containers if any
    docker-compose down || true

    # Start the services
    docker-compose up -d

    if [[ $? -ne 0 ]]; then
        log_error "Failed to start services"
        exit 1
    fi

    # Wait for services to be ready
    log_info "Waiting for services to be ready..."
    sleep 10

    # Check if the gateway is healthy
    if curl -f http://localhost:8080/health > /dev/null 2>&1; then
        log_success "Gateway is healthy"
    else
        log_error "Gateway health check failed"
        docker-compose logs forgecode-gateway
        exit 1
    fi

    log_success "Application deployed successfully"
}

# Function to display deployment information
show_deployment_info() {
    log_info "Deployment Information:"
    echo ""
    echo "  Application: ${PROJECT_NAME}"
    echo "  Environment: ${ENVIRONMENT}"
    echo "  Gateway URL: http://localhost:8080"
    echo "  Web UI: http://localhost:8080/static"
    echo "  Health Check: http://localhost:8080/health"
    echo ""
    echo "  Docker Compose Services:"
    echo "    - forgecode-gateway: Main application"
    echo "    - redis: Session storage and caching"
    echo "    - nginx: Reverse proxy and SSL termination"
    echo ""
    echo "  Useful Commands:"
    echo "    docker-compose logs -f forgecode-gateway"
    echo "    docker-compose restart forgecode-gateway"
    echo "    docker-compose down"
    echo ""

    log_success "Deployment completed!"
}

# Function to push images to registry (if registry is configured)
push_to_registry() {
    if [[ -n "$DOCKER_REGISTRY" ]]; then
        log_info "Pushing images to registry..."

        docker push "${DOCKER_REGISTRY}/${PROJECT_NAME}:latest"
        docker push "${DOCKER_REGISTRY}/${PROJECT_NAME}:${ENVIRONMENT}"

        if [[ $? -ne 0 ]]; then
            log_error "Failed to push images to registry"
            exit 1
        fi

        log_success "Images pushed to registry successfully"
    fi
}

# Main deployment function
main() {
    log_info "Starting deployment of ${PROJECT_NAME} (${ENVIRONMENT})..."

    check_prerequisites
    build_application
    build_docker_images
    setup_configuration
    deploy_application

    if [[ "$ENVIRONMENT" != "development" ]]; then
        push_to_registry
    fi

    show_deployment_info
}

# Handle script arguments
case "${1:-}" in
    "production")
        ENVIRONMENT="production"
        log_warning "Deploying to production environment"
        ;;
    "staging")
        ENVIRONMENT="staging"
        ;;
    "development"|"")
        ENVIRONMENT="development"
        ;;
    "-h"|"--help")
        echo "Usage: $0 [environment]"
        echo ""
        echo "Environments:"
        echo "  development (default)"
        echo "  staging"
        echo "  production"
        echo ""
        echo "Examples:"
        echo "  $0                    # Deploy to development"
        echo "  $0 staging            # Deploy to staging"
        echo "  $0 production         # Deploy to production"
        exit 0
        ;;
    *)
        log_error "Unknown environment: $1"
        echo "Use -h or --help for usage information"
        exit 1
        ;;
esac

# Run the main function
main