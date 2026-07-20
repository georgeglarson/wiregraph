# wiregraph

Real-time network traffic visualization. Hosts become nodes, traffic becomes flow, attacks become visual patterns.

Rust backend with an embedded web dashboard. Reuses [netgrep](https://github.com/georgeglarson/netgrep) modules for packet parsing.

```
┌─────────────────────────────┐     Browser
│  Rust Backend (localhost)   │ ◄──────────────
│                             │
│  netgrep capture/parsing    │  GET /          → embedded dashboard (HTML/JS/CSS)
│  → topology aggregation     │  GET /api/topology
│  → connection matrix        │  GET /api/events
│  → protocol breakdown       │  GET /api/stats
│  → activity timeline        │
│  → tiny_http JSON API       │
└─────────────────────────────┘
```

## Features

- **Top talkers** — ranked host list with traffic volume, packet count, protocol tags
- **Connection matrix** — host-to-host traffic heatmap with color-coded intensity
- **Protocol breakdown** — horizontal bar chart by bytes (TCP, HTTP, TLS, DNS, SSH, DHCP, etc.)
- **Activity timeline** — stacked area chart showing traffic over time by protocol
- **Live capture** — watch your network in real time (requires root/CAP_NET_RAW)
- **Pcap replay** — load any .pcap/.pcapng file, no privileges needed
- **BPF filtering** — standard Berkeley Packet Filter expressions
- **Pcap export** — export captured traffic from the browser

## Status & provenance

Built early 2026 as an AI-assisted portfolio project: a working demo of a live traffic visualizer, not a supported product. The design shifted along the way (an early 3D force-graph gave way to this 2D analytics dashboard), and it never got a production shakedown.

Where it stands after a July 2026 cleanup pass:

- Builds clean and passes its 108-test suite on current stable Rust, with a clean `clippy -D warnings`, `fmt`, and CI on stable + 1.91.
- Verified end to end: run it against a pcap and it serves the dashboard plus a JSON API (topology, stats, events) with real parsed data.
- Reuses [netgrep](https://github.com/georgeglarson/netgrep) for packet parsing, pinned to a current commit.
- Not actively maintained. Issues and PRs are welcome, no support promised.

Read it as a reference implementation and a code sample, not something to drop into production.

## Prerequisites

- Rust 1.91+
- libpcap-dev / libpcap

## Quick Start

### Demo mode (pcap file, no root needed)

```bash
./scripts/demo.sh sample.pcap
```

### Live capture

```bash
sudo ./scripts/run.sh eth0
```

### Manual

```bash
cd backend
cargo build --release
./target/release/wiregraph-backend --file ../sample.pcap
# open http://localhost:9877
```

## API

| Endpoint | Method | Returns |
|----------|--------|---------|
| `/` | GET | Embedded dashboard |
| `/api/topology` | GET | `{ nodes: [...], edges: [...] }` |
| `/api/events?since={ts}` | GET | `[...recent PacketEvents]` |
| `/api/stats` | GET | `{ total_packets, hosts, pps, ... }` |

## CLI

```
wiregraph-backend [OPTIONS]
  -i, --interface <NAME>    Live capture interface
  -f, --file <PATH>         Load pcap file
  -p, --port <PORT>         HTTP port [default: 9877]
  --filter <BPF>            BPF filter expression
```

## License

MIT
