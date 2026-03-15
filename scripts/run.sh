#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Ensure cargo is in PATH
[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"

PORT="${PORT:-9877}"
INTERFACE="${1:-}"
BPF_FILTER="${2:-}"

echo "=== wiregraph ==="
echo ""

# Build backend
echo "[1/3] building backend..."
cd "$ROOT_DIR/backend"
cargo build --release 2>&1 | tail -1
BACKEND_BIN="$ROOT_DIR/backend/target/release/wiregraph-backend"

# Build frontend
echo "[2/3] building frontend..."
cd "$ROOT_DIR/frontend"
npm install --silent 2>/dev/null
npm run build 2>&1 | tail -1

# Launch
echo "[3/3] launching..."
echo ""

BACKEND_ARGS=(--port "$PORT")
if [ -n "$INTERFACE" ]; then
    BACKEND_ARGS+=(--interface "$INTERFACE")
fi
if [ -n "$BPF_FILTER" ]; then
    BACKEND_ARGS+=(--filter "$BPF_FILTER")
fi

# Start backend in background
"$BACKEND_BIN" "${BACKEND_ARGS[@]}" &
BACKEND_PID=$!

# Give backend a moment to bind
sleep 1

# Start frontend
cd "$ROOT_DIR/frontend"
mystral run dist/wiregraph.js --width 1920 --height 1080 --title "wiregraph" &
FRONTEND_PID=$!

# Cleanup on exit
cleanup() {
    echo ""
    echo "shutting down..."
    kill "$FRONTEND_PID" 2>/dev/null || true
    kill "$BACKEND_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

wait "$FRONTEND_PID"
