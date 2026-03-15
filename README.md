# wiregraph

GPU-rendered network traffic visualizer. Hosts become nodes, traffic becomes flow, attacks become visual patterns.

Built on [Mystral Native](https://github.com/nicholasgasior/mystral) (WebGPU JS runtime) + a Rust capture backend that reuses [netgrep](https://github.com/georgeglarson/netgrep) modules for packet parsing.

```
┌─────────────────────────────┐     HTTP polling      ┌──────────────────────────────┐
│  Rust Backend (localhost)   │ ◄──────────────────── │  Mystral Native Frontend     │
│                             │                        │                              │
│  netgrep capture/parsing    │  GET /api/topology     │  Three.js + WebGPU           │
│  → topology aggregation     │  GET /api/events       │  d3-force-3d layout          │
│  → tiny_http JSON API       │  GET /api/stats        │  InstancedMesh rendering     │
│                             │                        │  Canvas 2D HUD overlay       │
└─────────────────────────────┘                        └──────────────────────────────┘
```

## Features

- **3D force-directed graph** — hosts as icosahedrons, connections as colored edges
- **Live capture** — watch your network in real time (requires root/CAP_NET_RAW)
- **Pcap replay** — load any .pcap/.pcapng file, no privileges needed
- **Packet particles** — animated sprites flow along edges showing live traffic
- **Protocol coloring** — HTTP=cyan, TLS=green, DNS=yellow, SSH=orange, UDP=purple
- **Node sizing** — logarithmic scaling by traffic volume
- **Subnet grouping** — local vs public IP visual distinction
- **HUD overlay** — stats panel, selected node details, protocol legend
- **Orbit controls** — drag to rotate, scroll to zoom, click to select

## Prerequisites

- Rust 1.91+ (for backend)
- Node.js 20+ (for frontend build)
- [Mystral Native](https://github.com/nicholasgasior/mystral) (for GPU rendering)
- libpcap-dev / libpcap (for packet capture)

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
# Backend
cd backend
cargo build --release
./target/release/wiregraph-backend --file ../sample.pcap

# Frontend (separate terminal)
cd frontend
npm install
npm run build
mystral run dist/wiregraph.js --width 1920 --height 1080 --title wiregraph
```

## Backend API

| Endpoint | Method | Returns |
|----------|--------|---------|
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

## Controls

| Key | Action |
|-----|--------|
| Drag | Rotate camera |
| Scroll | Zoom |
| Click | Select node |
| Space | Pause/resume |
| R | Reset camera |
| F | Focus selected node |

## License

MIT
