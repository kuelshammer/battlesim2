#!/bin/bash

# Kill any existing Next.js dev servers
pkill -f "next dev"

# Start the dev server on port 3000
npm run dev -- --port 3000 &

# Wait for the server to start
sleep 2

# Check if the server is running
if curl -I http://localhost:3000 >/dev/null 2>&1; then
    echo "Next.js dev server started successfully on port 3000"
    echo "You can access it at http://localhost:3000"
else
    echo "Failed to start Next.js dev server on port 3000"
    echo "Please check for any port conflicts"
fi

# Keep the script running to keep the server alive
wait