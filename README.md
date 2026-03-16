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
