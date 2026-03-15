#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"

PORT="${PORT:-9877}"

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
echo "[1/2] building backend..."
cd "$ROOT_DIR/backend"
cargo build --release 2>&1 | tail -1
BACKEND_BIN="$ROOT_DIR/backend/target/release/wiregraph-backend"

echo "[2/2] launching..."
echo ""

BACKEND_ARGS=(--port "$PORT")

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

# Start backend
"$BACKEND_BIN" "${BACKEND_ARGS[@]}" &
BACKEND_PID=$!

sleep 1

# Open browser
URL="http://127.0.0.1:${PORT}"
echo "opening $URL"
if command -v xdg-open &>/dev/null; then
    xdg-open "$URL" 2>/dev/null &
elif command -v open &>/dev/null; then
    open "$URL" &
else
    echo "open $URL in your browser"
fi

cleanup() {
    echo ""
    echo "shutting down..."
    kill "$BACKEND_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

wait "$BACKEND_PID"
