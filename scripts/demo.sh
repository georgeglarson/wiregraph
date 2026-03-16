#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"

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
echo "building..."
cd "$ROOT_DIR/backend"
cargo build --release 2>&1 | tail -1
BACKEND_BIN="$ROOT_DIR/backend/target/release/wiregraph-backend"

echo "launching..."
echo ""

# Kill any stale process on the port
lsof -ti:"$PORT" 2>/dev/null | xargs kill 2>/dev/null || true

# Start backend
"$BACKEND_BIN" --file "$PCAP_FILE" --port "$PORT" &
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
