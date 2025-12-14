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
      echo "🔪 Killing process tree for PID $PID"
      pkill -TERM -P "$PID" 2>/dev/null || true
      kill "$PID" 2>/dev/null || true
    fi
  done

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
  cd "$BACKEND_DIR"
  CARGO_INCREMENTAL=0 cargo build --release
)

echo "🚀 Starting Rust backend on port 3001..."
(
  cd "$BACKEND_DIR"
  NSE_MODE=server NSE_PORT=3001 ./target/release/nse-analyzer
) &
CHILD_PIDS+=($!)

#######################################
# Frontend (Next.js)
#######################################

echo
echo "📦 Building frontend..."
(
  cd "$FRONTEND_DIR"
  npm run build
)

echo "🚀 Starting frontend on port 3000..."
(
  cd "$FRONTEND_DIR"
  npm run dev
) &
CHILD_PIDS+=($!)

echo
echo "✅ Backend + Frontend running"
echo "Press Ctrl+C to stop everything"

# Keep script alive
wait
