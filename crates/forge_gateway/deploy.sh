#!/bin/bash

# Forgecode Gateway Deployment Script
set -e

echo "🚀 Deploying Forgecode Gateway..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose is not installed. Please install docker-compose first."
    exit 1
fi

# Build the image
echo "📦 Building Docker image..."
docker-compose build

# Stop existing containers if they exist
echo "🛑 Stopping existing containers..."
docker-compose down || true

# Start the service
echo "🚀 Starting Forgecode Gateway..."
docker-compose up -d

# Wait for service to be healthy
echo "⏳ Waiting for service to be healthy..."
for i in {1..30}; do
    if docker-compose ps | grep -q "Up (healthy)"; then
        echo "✅ Service is healthy!"
        break
    fi
    echo "⏳ Waiting for service to become healthy... ($i/30)"
    sleep 2

    if [ $i -eq 30 ]; then
        echo "❌ Service did not become healthy within 60 seconds"
        docker-compose logs
        exit 1
    fi
done

echo "🎉 Forgecode Gateway deployed successfully!"
echo "🌐 Access the gateway at: http://localhost:8080"
echo "📊 Health check: http://localhost:8080/health"
echo ""
echo "📋 Useful commands:"
echo "  docker-compose logs -f    # View logs"
echo "  docker-compose down        # Stop the service"
echo "  docker-compose restart     # Restart the service"