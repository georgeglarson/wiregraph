#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

REAL_HOME="${SUDO_USER:+$(eval echo "~$SUDO_USER")}"
REAL_HOME="${REAL_HOME:-$HOME}"
if [[ -d "$REAL_HOME/.cargo/bin" ]]; then
    export PATH="$REAL_HOME/.cargo/bin:$PATH"
    export RUSTUP_HOME="$REAL_HOME/.rustup"
    export CARGO_HOME="$REAL_HOME/.cargo"
fi

PORT="${PORT:-9877}"
BACKEND_BIN="$ROOT_DIR/backend/target/release/wiregraph-backend"

usage() {
    echo "usage: $0 [interface|pcap-file] [bpf-filter]"
    echo ""
    echo "  $0                    # interactive interface selection"
    echo "  $0 1                  # capture on interface #1"
    echo "  $0 eth0               # capture on eth0"
    echo "  $0 1,3                # capture on interfaces #1 and #3"
    echo "  $0 eth0,wlan0         # capture on multiple interfaces"
    echo "  $0 ../sample.pcap     # replay pcap file"
    echo "  $0 eth0 'port 80'     # live capture with BPF filter"
    echo "  $0 -L                 # list interfaces and exit"
    exit 1
}

[[ "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage

echo "=== wiregraph ==="
echo ""

# Build backend
echo "[1/2] building backend..."
cd "$ROOT_DIR/backend"
cargo build --release 2>&1 | tail -1

# Handle -L
if [[ "${1:-}" == "-L" ]]; then
    "$BACKEND_BIN" --list-interfaces
    exit 0
fi

# Get interface list (as "name" per line) for resolution
get_iface_names() {
    "$BACKEND_BIN" --list-interfaces 2>&1 | sed -n 's/^ *[0-9]*) \([^ ]*\).*/\1/p'
}

# Resolve a token (number or name) to an interface name
resolve_iface() {
    local token="$1"
    # If it's a number, map to the Nth interface
    if [[ "$token" =~ ^[0-9]+$ ]]; then
        local name
        name=$(get_iface_names | sed -n "${token}p")
        if [[ -z "$name" ]]; then
            echo "error: interface #${token} not found" >&2
            return 1
        fi
        echo "$name"
    else
        echo "$token"
    fi
}

TARGET="${1:-}"
BPF_FILTER="${2:-}"

# If no target, interactive selection
if [[ -z "$TARGET" ]]; then
    "$BACKEND_BIN" --list-interfaces
    echo ""
    read -rp "Select interfaces (numbers or names, comma-separated): " TARGET
    if [[ -z "$TARGET" ]]; then
        echo "error: no interfaces selected"
        exit 1
    fi
fi

echo "[2/2] launching..."
echo ""

BACKEND_ARGS=(--port "$PORT")

if [[ -f "$TARGET" ]]; then
    BACKEND_ARGS+=(--file "$TARGET")
else
    # Resolve each comma-separated token
    RESOLVED=""
    IFS=',' read -ra TOKENS <<< "$TARGET"
    for tok in "${TOKENS[@]}"; do
        tok=$(echo "$tok" | xargs)  # trim whitespace
        [[ -z "$tok" ]] && continue
        name=$(resolve_iface "$tok") || exit 1
        if [[ -n "$RESOLVED" ]]; then
            RESOLVED="${RESOLVED},${name}"
        else
            RESOLVED="$name"
        fi
    done
    if [[ -z "$RESOLVED" ]]; then
        echo "error: no valid interfaces"
        exit 1
    fi
    BACKEND_ARGS+=(--interface "$RESOLVED")
fi

if [[ -n "$BPF_FILTER" ]]; then
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
