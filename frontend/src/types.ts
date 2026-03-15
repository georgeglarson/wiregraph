export interface Node {
  ip: string;
  subnet: string;
  bytes_sent: number;
  bytes_recv: number;
  packet_count: number;
  protocols: string[];
  is_local: boolean;
  last_seen: number;
  // Layout state (set by d3-force)
  x?: number;
  y?: number;
  z?: number;
  vx?: number;
  vy?: number;
  vz?: number;
  index?: number;
}

export interface Edge {
  source: string;
  target: string;
  dst_port: number;
  protocol: string;
  bytes: number;
  packets: number;
  active: boolean;
  last_seen: number;
}

export interface PacketEvent {
  timestamp: number;
  source: string;
  target: string;
  bytes: number;
  protocol: string;
}

export interface Stats {
  total_packets: number;
  total_bytes: number;
  host_count: number;
  edge_count: number;
  packets_per_second: number;
  capture_duration: number;
}

export interface TopologyResponse {
  nodes: Node[];
  edges: Edge[];
}

export const PROTOCOL_COLORS: Record<string, number> = {
  HTTP: 0x00ffff,
  TLS: 0x00ff88,
  DNS: 0xffff00,
  SSH: 0xff8800,
  UDP: 0xaa44ff,
  TCP: 0x4488ff,
  ICMP: 0xff4444,
  DHCP: 0x88ff44,
  NTP: 0xff88ff,
  SMTP: 0xff6644,
  OTHER: 0x888888,
};
