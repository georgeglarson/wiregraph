use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::time::SystemTime;

use crate::models::*;

const EVENT_BUFFER_SECS: f64 = 5.0;
const ACTIVE_TIMEOUT_SECS: f64 = 3.0;

pub struct Topology {
    pub nodes: HashMap<IpAddr, Node>,
    pub edges: HashMap<EdgeKey, Edge>,
    pub events: VecDeque<PacketEvent>,
    pub total_packets: u64,
    pub total_bytes: u64,
    start_time: f64,
}

impl Topology {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            events: VecDeque::new(),
            total_packets: 0,
            total_bytes: 0,
            start_time: now_secs(),
        }
    }

    pub fn ingest(
        &mut self,
        src_ip: IpAddr,
        dst_ip: IpAddr,
        _src_port: Option<u16>,
        dst_port: Option<u16>,
        protocol: &str,
        bytes: u64,
        timestamp: Option<SystemTime>,
    ) {
        let ts = timestamp
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or_else(now_secs);

        self.total_packets += 1;
        self.total_bytes += bytes;

        // Update source node
        let src_node = self.nodes.entry(src_ip).or_insert_with(|| Node::new(src_ip));
        src_node.bytes_sent += bytes;
        src_node.packet_count += 1;
        src_node.protocols.insert(protocol.to_string());
        src_node.last_seen = ts;

        // Update destination node
        let dst_node = self.nodes.entry(dst_ip).or_insert_with(|| Node::new(dst_ip));
        dst_node.bytes_recv += bytes;
        dst_node.packet_count += 1;
        dst_node.protocols.insert(protocol.to_string());
        dst_node.last_seen = ts;

        // Update edge (grouped by dst port)
        let dp = dst_port.unwrap_or(0);
        let edge_key = EdgeKey {
            source: src_ip,
            target: dst_ip,
            dst_port: dp,
        };
        let edge = self.edges.entry(edge_key).or_insert_with(|| {
            Edge::new(src_ip, dst_ip, dp, protocol)
        });
        edge.bytes += bytes;
        edge.packets += 1;
        edge.active = true;
        edge.last_seen = ts;

        // Append event
        let event = PacketEvent {
            timestamp: ts,
            source: src_ip,
            target: dst_ip,
            bytes,
            protocol: protocol.to_string(),
        };
        self.events.push_back(event);

        // Prune old events
        let cutoff = ts - EVENT_BUFFER_SECS;
        while self.events.front().is_some_and(|e| e.timestamp < cutoff) {
            self.events.pop_front();
        }
    }

    pub fn mark_inactive(&mut self) {
        let now = now_secs();
        for edge in self.edges.values_mut() {
            if now - edge.last_seen > ACTIVE_TIMEOUT_SECS {
                edge.active = false;
            }
        }
    }

    pub fn topology_response(&self) -> TopologyResponse {
        TopologyResponse {
            nodes: self.nodes.values().cloned().collect(),
            edges: self.edges.values().cloned().collect(),
        }
    }

    pub fn events_since(&self, since: f64) -> Vec<PacketEvent> {
        self.events.iter().filter(|e| e.timestamp > since).cloned().collect()
    }

    pub fn stats(&self) -> Stats {
        let elapsed = now_secs() - self.start_time;
        let pps = if elapsed > 0.0 {
            self.total_packets as f64 / elapsed
        } else {
            0.0
        };
        Stats {
            total_packets: self.total_packets,
            total_bytes: self.total_bytes,
            host_count: self.nodes.len(),
            edge_count: self.edges.len(),
            packets_per_second: pps,
            capture_duration: elapsed,
        }
    }
}

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn ip(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(a, b, c, d))
    }

    #[test]
    fn ingest_creates_nodes_and_edges() {
        let mut topo = Topology::new();
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(12345), Some(80), "TCP", 100, None);

        assert_eq!(topo.nodes.len(), 2);
        assert_eq!(topo.edges.len(), 1);
        assert_eq!(topo.total_packets, 1);
        assert_eq!(topo.total_bytes, 100);

        let src = &topo.nodes[&ip(10, 0, 0, 1)];
        assert_eq!(src.bytes_sent, 100);
        assert_eq!(src.bytes_recv, 0);
        assert!(src.is_local);

        let dst = &topo.nodes[&ip(10, 0, 0, 2)];
        assert_eq!(dst.bytes_recv, 100);
        assert_eq!(dst.bytes_sent, 0);
    }

    #[test]
    fn multiple_packets_accumulate() {
        let mut topo = Topology::new();
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(12345), Some(80), "TCP", 100, None);
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(12345), Some(80), "TCP", 200, None);

        assert_eq!(topo.nodes.len(), 2);
        assert_eq!(topo.edges.len(), 1);
        assert_eq!(topo.total_packets, 2);
        assert_eq!(topo.total_bytes, 300);

        let edge = topo.edges.values().next().unwrap();
        assert_eq!(edge.bytes, 300);
        assert_eq!(edge.packets, 2);
    }

    #[test]
    fn different_dst_ports_create_separate_edges() {
        let mut topo = Topology::new();
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(12345), Some(80), "TCP", 100, None);
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(12345), Some(443), "TCP", 200, None);

        assert_eq!(topo.edges.len(), 2);
    }

    #[test]
    fn events_since_filters_correctly() {
        let mut topo = Topology::new();
        let t1 = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000);
        let t2 = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1001);
        let t3 = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1002);

        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(1), Some(80), "TCP", 10, Some(t1));
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(1), Some(80), "TCP", 20, Some(t2));
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(1), Some(80), "TCP", 30, Some(t3));

        let events = topo.events_since(1000.5);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn topology_response_contains_all() {
        let mut topo = Topology::new();
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(1), Some(80), "TCP", 100, None);
        topo.ingest(ip(192, 168, 1, 1), ip(8, 8, 8, 8), Some(2), Some(53), "UDP", 50, None);

        let resp = topo.topology_response();
        assert_eq!(resp.nodes.len(), 4);
        assert_eq!(resp.edges.len(), 2);
    }

    #[test]
    fn stats_reflect_state() {
        let mut topo = Topology::new();
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(1), Some(80), "TCP", 100, None);

        let stats = topo.stats();
        assert_eq!(stats.total_packets, 1);
        assert_eq!(stats.total_bytes, 100);
        assert_eq!(stats.host_count, 2);
        assert_eq!(stats.edge_count, 1);
    }

    #[test]
    fn protocols_tracked_on_nodes() {
        let mut topo = Topology::new();
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 2), Some(1), Some(80), "TCP", 100, None);
        topo.ingest(ip(10, 0, 0, 1), ip(10, 0, 0, 3), Some(2), Some(53), "UDP", 50, None);

        let node = &topo.nodes[&ip(10, 0, 0, 1)];
        assert!(node.protocols.contains("TCP"));
        assert!(node.protocols.contains("UDP"));
    }

    #[test]
    fn public_ip_not_local() {
        let mut topo = Topology::new();
        topo.ingest(ip(10, 0, 0, 1), ip(8, 8, 8, 8), Some(1), Some(53), "UDP", 50, None);

        assert!(topo.nodes[&ip(10, 0, 0, 1)].is_local);
        assert!(!topo.nodes[&ip(8, 8, 8, 8)].is_local);
    }
}
