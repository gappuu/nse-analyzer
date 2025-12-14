#!/bin/bash

set -e

PORTS=(3000 3001)
CHILD_PIDS=()

BACKEND_DIR="$HOME/Desktop/nse-analyzer/backend"
FRONTEND_DIR="$HOME/Desktop/nse-analyzer/frontend"

cleanup() {
  echo
  echo "🛑 Stopping services..."
  for PID in "${CHILD_PIDS[@]}"; do
    if kill -0 "$PID" 2>/dev/null; then
      echo "🔪 Killing PID $PID"
      kill "$PID"
    fi
  done
  wait
  echo "✅ Services stopped"
  exit 0
}

trap cleanup SIGINT SIGTERM EXIT

kill_port() {
  lsof -ti tcp:$1 | xargs -r kill
}

echo "🧹 Freeing ports 3000 and 3001..."
for PORT in "${PORTS[@]}"; do
  kill_port "$PORT"
done

#######################################
# Backend (Rust)
#######################################

echo
echo "🦀 Building Rust backend (release, no incremental)..."
(
  cd "$BACKEND_DIR" || exit 1
  CARGO_INCREMENTAL=0 cargo build --release
)

echo "🚀 Running Rust backend on port 3001..."
(
  cd "$BACKEND_DIR" || exit 1
  NSE_MODE=server NSE_PORT=3001 \
    ./target/release/$(basename "$BACKEND_DIR")
) &
CHILD_PIDS+=($!)

#######################################
# Frontend (Node)
#######################################

echo
echo "📦 Building frontend..."
(
  cd "$FRONTEND_DIR" || exit 1
  npm run build
)

echo "🚀 Starting frontend on port 3000..."
(
  cd "$FRONTEND_DIR" || exit 1
  npm run dev
) &
CHILD_PIDS+=($!)

echo
echo "✅ All services running (Ctrl+C to stop)"

wait
