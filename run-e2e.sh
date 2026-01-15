#!/bin/bash
# run-e2e.sh

echo "Starting Next.js server..."
npm run dev -- --port 3000 > server.log 2>&1 &
SERVER_PID=$!

cleanup() {
  echo "Stopping server (PID: $SERVER_PID)..."
  kill $SERVER_PID
}

trap cleanup EXIT

echo "Waiting for server to be ready..."
timeout=60
while ! curl -s http://localhost:3000 > /dev/null; do
  sleep 1
  timeout=$((timeout-1))
  if [ $timeout -le 0 ]; then
    echo "Server failed to start"
    exit 1
  fi
done
echo "Server is ready!"

echo "Running E2E tests: $@"
E2E_BASE_URL=http://localhost:3000 npm run test:e2e -- "$@"
EXIT_CODE=$?

exit $EXIT_CODE
