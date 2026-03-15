#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Ensure cargo and mystral are in PATH
[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"
[[ -d "$HOME/.mystral" ]] && export PATH="$PATH:$HOME/.mystral"

# Mystral/SDL needs X11 (XWayland) on Wayland sessions
if [[ "${XDG_SESSION_TYPE:-}" == "wayland" ]]; then
    export SDL_VIDEODRIVER=x11
fi

PORT="${PORT:-9877}"
WIDTH="${WIDTH:-1280}"
HEIGHT="${HEIGHT:-720}"

usage() {
    echo "usage: $0 <interface|pcap-file> [bpf-filter]"
    echo ""
    echo "  $0 eth0              # live capture on eth0 (needs root)"
    echo "  $0 ../sample.pcap    # replay pcap file"
    echo "  $0 eth0 'port 80'    # live capture with BPF filter"
    exit 1
}

[[ $# -lt 1 ]] && usage

TARGET="$1"
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

# Detect whether target is a file or an interface name
if [ -f "$TARGET" ]; then
    BACKEND_ARGS+=(--file "$TARGET")
else
    BACKEND_ARGS+=(--interface "$TARGET")
fi

if [ -n "$BPF_FILTER" ]; then
    BACKEND_ARGS+=(--filter "$BPF_FILTER")
fi

# Kill any stale process on the port
lsof -ti:"$PORT" 2>/dev/null | xargs kill 2>/dev/null || true

# Start backend in background
"$BACKEND_BIN" "${BACKEND_ARGS[@]}" &
BACKEND_PID=$!

# Give backend a moment to bind
sleep 1

# Start frontend
cd "$ROOT_DIR/frontend"
mystral run dist/wiregraph.js --width "$WIDTH" --height "$HEIGHT" --title "wiregraph" &
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
