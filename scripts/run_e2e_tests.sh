#!/bin/bash
set -e

echo "Building AI-Nexus..."
cargo build --workspace

echo "Starting AI-Nexus daemon..."
# Provide a dummy key so it doesn't immediately exit
export GEMINI_API_KEY="dummy_key_for_test"

# Run in background and pipe input to avoid reading from stdin which could block or exit immediately
cargo run --bin ai-nexus < /dev/null > /tmp/ai-nexus.log 2>&1 &
DAEMON_PID=$!

echo "Waiting for Dashboard API on port 3000..."
max_attempts=30
attempt=0
while ! curl -s http://localhost:3000/api/dashboard/stats > /dev/null; do
    attempt=$((attempt+1))
    if [ $attempt -gt $max_attempts ]; then
        echo "Timeout waiting for API server. Logs:"
        cat /tmp/ai-nexus.log
        kill $DAEMON_PID
        exit 1
    fi
    sleep 1
done

echo "API server is up. Running joint tests..."

# Run the joint tests
if cargo test --package ainexus-test joint_e2e -- --ignored; then
    echo "E2E tests passed successfully."
    RET=0
else
    echo "E2E tests failed."
    RET=1
fi

echo "Cleaning up daemon process (PID: $DAEMON_PID)..."
kill $DAEMON_PID || true

exit $RET
