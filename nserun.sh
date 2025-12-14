#!/bin/bash

set -euo pipefail

# ==============================
# CONFIG
# ==============================
BACKEND_DIR="$HOME/Desktop/nse-analyzer/backend"
FRONTEND_DIR="$HOME/Desktop/nse-analyzer/frontend"
BACKEND_PORT=3001
FRONTEND_PORT=3000
LOG_DIR="$HOME/Desktop/nse-analyzer/logs"

mkdir -p "$LOG_DIR"
BACKEND_LOG="$LOG_DIR/backend.log"
FRONTEND_LOG="$LOG_DIR/frontend.log"

CHILD_PIDS=()

# ==============================
# FUNCTIONS
# ==============================
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

cleanup() {
    log "🛑 Stopping services..."

    for PID in "${CHILD_PIDS[@]}"; do
        if kill -0 "$PID" 2>/dev/null; then
            log "🔪 Killing process tree for PID $PID"
            pkill -TERM -P "$PID" 2>/dev/null || true
            kill "$PID" 2>/dev/null || true
        fi
    done

    # Delete log files
    if [[ -f "$BACKEND_LOG" ]]; then
        rm -f "$BACKEND_LOG"
        log "🗑️ Deleted backend log"
    fi
    if [[ -f "$FRONTEND_LOG" ]]; then
        rm -f "$FRONTEND_LOG"
        log "🗑️ Deleted frontend log"
    fi

    log "✅ Services stopped"
    exit 0
}

trap cleanup SIGINT SIGTERM EXIT

kill_port() {
    local PORT=$1
    local PIDS
    PIDS=$(lsof -ti tcp:"$PORT" || true)
    if [[ -n "$PIDS" ]]; then
        log "🧹 Killing processes on port $PORT: $PIDS"
        echo "$PIDS" | xargs -r kill
    fi
}

check_directory() {
    local DIR=$1
    if [[ ! -d "$DIR" ]]; then
        log "❌ Directory not found: $DIR"
        exit 1
    fi
}

healthcheck() {
    local URL=$1
    local RETRIES=10
    local COUNT=0
    log "🔎 Checking health for $URL"
    until curl -s "$URL" >/dev/null; do
        COUNT=$((COUNT+1))
        if (( COUNT > RETRIES )); then
            log "❌ Health check failed for $URL"
            exit 1
        fi
        sleep 1
    done
    log "✅ $URL is up"
}

# ==============================
# VALIDATIONS
# ==============================
check_directory "$BACKEND_DIR"
check_directory "$FRONTEND_DIR"

kill_port "$BACKEND_PORT"
kill_port "$FRONTEND_PORT"

# ==============================
# BACKEND
# ==============================
log "🦀 Building Rust backend (release, no incremental)..."
(
    cd "$BACKEND_DIR"
    CARGO_INCREMENTAL=0 cargo build --release >>"$BACKEND_LOG" 2>&1
)

log "🚀 Starting Rust backend on port $BACKEND_PORT..."
(
    cd "$BACKEND_DIR"
    NSE_MODE=server NSE_PORT="$BACKEND_PORT" ./target/release/nse-analyzer >>"$BACKEND_LOG" 2>&1
) &
CHILD_PIDS+=($!)

# Wait for backend to be up
healthcheck "http://127.0.0.1:$BACKEND_PORT/api/securities"

# ==============================
# FRONTEND
# ==============================
log "📦 Building frontend..."
(
    cd "$FRONTEND_DIR"
    npm run build >>"$FRONTEND_LOG" 2>&1
)

log "🚀 Starting frontend on port $FRONTEND_PORT..."
(
    cd "$FRONTEND_DIR"
    npm run dev >>"$FRONTEND_LOG" 2>&1
) &
CHILD_PIDS+=($!)

log "✅ Backend + Frontend running"
log "Logs: backend=$BACKEND_LOG frontend=$FRONTEND_LOG"
log "Press Ctrl+C to stop everything"

# ==============================
# KEEP SCRIPT ALIVE
# ==============================
wait
