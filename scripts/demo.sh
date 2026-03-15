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
PCAP_FILE="${1:-$ROOT_DIR/sample.pcap}"

if [ ! -f "$PCAP_FILE" ]; then
    echo "error: pcap file not found: $PCAP_FILE"
    echo "usage: $0 [path/to/file.pcap]"
    exit 1
fi

echo "=== wiregraph demo ==="
echo "pcap: $PCAP_FILE"
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

# Start backend with pcap file
"$BACKEND_BIN" --file "$PCAP_FILE" --port "$PORT" &
BACKEND_PID=$!

sleep 1

# Start frontend
cd "$ROOT_DIR/frontend"
mystral run dist/wiregraph.js --width 1920 --height 1080 --title "wiregraph — demo" &
FRONTEND_PID=$!

cleanup() {
    echo ""
    echo "shutting down..."
    kill "$FRONTEND_PID" 2>/dev/null || true
    kill "$BACKEND_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

wait "$FRONTEND_PID"
