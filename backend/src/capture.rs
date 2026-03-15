use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use netgrep::capture::PacketSource;
use netgrep::protocol::parse_packet;

use crate::models::classify_protocol;
use crate::topology::Topology;

fn process_packets(
    source: &mut PacketSource,
    topology: Arc<RwLock<Topology>>,
) -> Result<()> {
    let link_type = source.link_type();

    source.for_each_packet(|pkt| {
        if let Some(parsed) = parse_packet(pkt.data, link_type) {
            if let (Some(src_ip), Some(dst_ip)) = (parsed.src_ip, parsed.dst_ip) {
                let protocol = classify_protocol(parsed.transport, parsed.src_port, parsed.dst_port);
                let bytes = pkt.data.len() as u64;

                if let Ok(mut topo) = topology.write() {
                    topo.ingest(
                        src_ip,
                        dst_ip,
                        parsed.src_port,
                        parsed.dst_port,
                        protocol,
                        bytes,
                        Some(pkt.timestamp),
                    );
                }
            }
        }
        true
    })?;

    Ok(())
}

pub fn run_capture_file(
    path: &Path,
    bpf: Option<&str>,
    topology: Arc<RwLock<Topology>>,
) -> Result<()> {
    let mut source = PacketSource::from_file(path, bpf)?;
    process_packets(&mut source, topology)
}

pub fn run_capture_live(
    interface: Option<&str>,
    bpf: Option<&str>,
    topology: Arc<RwLock<Topology>>,
) -> Result<()> {
    let mut source = PacketSource::live(interface, 65535, true, bpf, None)?;
    process_packets(&mut source, topology)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_pcap() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sample.pcap")
    }

    #[test]
    fn capture_file_populates_topology() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        assert!(t.total_packets > 0, "should have captured packets");
        assert!(t.nodes.len() >= 2, "should have at least 2 hosts");
        assert!(!t.edges.is_empty(), "should have at least 1 edge");
    }

    #[test]
    fn capture_file_finds_expected_hosts() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        let ips: Vec<String> = t.nodes.keys().map(|ip| ip.to_string()).collect();

        assert!(ips.contains(&"192.168.1.10".to_string()), "missing local host");
        assert!(ips.contains(&"8.8.8.8".to_string()), "missing DNS server");
    }

    #[test]
    fn capture_file_classifies_protocols() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        let all_protocols: std::collections::HashSet<String> = t.edges.values()
            .map(|e| e.protocol.clone())
            .collect();

        // Sample pcap has HTTP, DNS, TLS, SSH traffic
        assert!(all_protocols.contains("HTTP"), "missing HTTP edges");
        assert!(all_protocols.contains("DNS"), "missing DNS edges");
    }

    #[test]
    fn capture_file_events_populated() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        assert!(!t.events.is_empty(), "events ring buffer should have entries");
    }

    #[test]
    fn capture_file_nonexistent_errors() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        let result = run_capture_file(Path::new("/nonexistent.pcap"), None, topo);
        assert!(result.is_err());
    }

    #[test]
    fn capture_file_bytes_consistent() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        let stats = t.stats();

        // Total bytes across all nodes (sent) should equal total_bytes
        let total_sent: u64 = t.nodes.values().map(|n| n.bytes_sent).sum();
        assert_eq!(total_sent, stats.total_bytes, "node bytes_sent should sum to total");
    }

    #[test]
    fn capture_file_packet_count_consistent() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        let total_edge_packets: u64 = t.edges.values().map(|e| e.packets).sum();
        assert_eq!(total_edge_packets, t.total_packets, "edge packets should sum to total");
    }
}
