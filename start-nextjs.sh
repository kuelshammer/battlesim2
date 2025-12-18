#!/bin/bash

# Function to ensure port 3000 is free
ensure_port_free() {
    echo "Ensuring port 3000 is free..."
    # Find and kill any process using port 3000
    lsof -ti:3000 | xargs -r kill -9 2>/dev/null
    sleep 1
}

# Function to start Next.js dev server
start_nextjs() {
    echo "Starting Next.js dev server on port 3000..."
    npm run dev -- --port 3000 &
    NEXTJS_PID=$!
    
    # Wait for server to start
    sleep 3
    
    # Check if server is running
    if curl -I http://localhost:3000 >/dev/null 2>&1; then
        echo "Next.js dev server started successfully on port 3000"
        echo "You can access it at http://localhost:3000"
        wait $NEXTJS_PID
    else
        echo "Failed to start Next.js dev server on port 3000"
        echo "Please check for any port conflicts"
        kill $NEXTJS_PID 2>/dev/null
    fi
}

# Main execution
ensure_port_free
start_nextjs