#!/bin/bash
set -e

echo "=== Amadeus Runner ==="

# Check for Java
if ! command -v java &> /dev/null; then
    echo "Error: Java is not installed."
    exit 1
fi

# Frontend
echo "Starting Frontend..."
cd amadeus-web
if [ ! -d "node_modules" ]; then
    echo "Installing frontend dependencies..."
    npm install
fi
# Run in background
npm run dev &
FRONTEND_PID=$!
cd ..

# Backend
echo "Starting Backend..."
cd amadeus-server
if command -v mvn &> /dev/null; then
    mvn spring-boot:run
else
    echo "Maven not found globally. Checking for wrapper..."
    if [ -f "./mvnw" ]; then
        ./mvnw spring-boot:run
    else
        echo "Error: Maven (mvn) not found and no wrapper present."
        echo "Please install Maven to run the backend."
        kill $FRONTEND_PID
        exit 1
    fi
fi

# Cleanup
trap "kill $FRONTEND_PID" EXIT

