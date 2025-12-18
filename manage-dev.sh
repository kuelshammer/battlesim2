#!/bin/bash

# Function to kill Next.js dev servers
kill_next_dev() {
    echo "Killing existing Next.js dev servers..."
    pkill -f "next dev"
    sleep 2
}

# Function to start Next.js dev server
start_next_dev() {
    echo "Starting Next.js dev server on port 3000..."
    npm run dev -- --port 3000 &
    sleep 3
    
    # Check if server started
    if curl -I http://localhost:3000 >/dev/null 2>&1; then
        echo "Next.js dev server started successfully on port 3000"
        echo "You can access it at http://localhost:3000"
    else
        echo "Failed to start Next.js dev server on port 3000"
        echo "Please check for any port conflicts"
    fi
}

# Kill existing servers
kill_next_dev

# Start new server
start_next_dev

# Keep the script running
wait