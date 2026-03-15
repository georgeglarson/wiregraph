use serde::Serialize;
use std::collections::HashSet;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub ip: IpAddr,
    pub subnet: String,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub packet_count: u64,
    pub protocols: HashSet<String>,
    pub is_local: bool,
    pub last_seen: f64,
}

impl Node {
    pub fn new(ip: IpAddr) -> Self {
        Self {
            ip,
            subnet: subnet_of(ip),
            bytes_sent: 0,
            bytes_recv: 0,
            packet_count: 0,
            protocols: HashSet::new(),
            is_local: is_local_ip(ip),
            last_seen: 0.0,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize)]
pub struct EdgeKey {
    pub source: IpAddr,
    pub target: IpAddr,
    pub dst_port: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    pub source: IpAddr,
    pub target: IpAddr,
    pub dst_port: u16,
    pub protocol: String,
    pub bytes: u64,
    pub packets: u64,
    pub active: bool,
    pub last_seen: f64,
}

impl Edge {
    pub fn new(source: IpAddr, target: IpAddr, dst_port: u16, protocol: &str) -> Self {
        Self {
            source,
            target,
            dst_port,
            protocol: protocol.to_string(),
            bytes: 0,
            packets: 0,
            active: true,
            last_seen: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PacketEvent {
    pub timestamp: f64,
    pub source: IpAddr,
    pub target: IpAddr,
    pub bytes: u64,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub total_packets: u64,
    pub total_bytes: u64,
    pub host_count: usize,
    pub edge_count: usize,
    pub packets_per_second: f64,
    pub capture_duration: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopologyResponse {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

fn subnet_of(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
        }
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", segments[0], segments[1], segments[2], segments[3])
        }
    }
}

fn is_local_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // ULA
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // link-local
        }
    }
}
