use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use anyhow::Result;
use netgrep::capture::{PacketData, PacketSource};
use netgrep::protocol::{parse_packet, LinkType};

use crate::models::classify_protocol;
use crate::packet_store::PacketStore;
use crate::topology::Topology;

struct BufferedPacket {
    data: Vec<u8>,
    timestamp: SystemTime,
}

fn ingest_packet(
    raw: &[u8],
    timestamp: SystemTime,
    link_type: LinkType,
    topology: &Arc<RwLock<Topology>>,
    store: &Arc<RwLock<PacketStore>>,
) {
    // Store raw packet for export
    if let Ok(mut s) = store.write() {
        s.add(raw, timestamp);
    }

    // Parse and ingest into topology
    if let Some(parsed) = parse_packet(raw, link_type) {
        if let (Some(src_ip), Some(dst_ip)) = (parsed.src_ip, parsed.dst_ip) {
            let protocol = classify_protocol(parsed.transport, parsed.src_port, parsed.dst_port);
            let bytes = raw.len() as u64;

            if let Ok(mut topo) = topology.write() {
                topo.ingest(src_ip, dst_ip, parsed.src_port, parsed.dst_port, protocol, bytes, None);
            }
        }
    }
}

pub fn run_capture_file(
    path: &Path,
    bpf: Option<&str>,
    topology: Arc<RwLock<Topology>>,
    store: Arc<RwLock<PacketStore>>,
) -> Result<()> {
    let mut source = PacketSource::from_file(path, bpf)?;
    let link_type = source.link_type();
    let mut packets: Vec<BufferedPacket> = Vec::new();

    source.for_each_packet(|pkt: PacketData| {
        packets.push(BufferedPacket {
            data: pkt.data.to_vec(),
            timestamp: pkt.timestamp,
        });
        true
    })?;

    if packets.is_empty() {
        eprintln!("no packets in pcap file");
        return Ok(());
    }

    let first_ts = packets[0].timestamp;
    let last_ts = packets[packets.len() - 1].timestamp;
    let capture_span = last_ts.duration_since(first_ts).unwrap_or(Duration::from_secs(1));
    let replay_secs = 8.0f64;
    let speedup = if capture_span.as_secs_f64() > 0.001 {
        capture_span.as_secs_f64() / replay_secs
    } else {
        0.001
    };

    eprintln!(
        "replaying {} packets over {:.0}s (original span: {:.2}s)",
        packets.len(), replay_secs, capture_span.as_secs_f64()
    );

    loop {
        if let Ok(mut topo) = topology.write() {
            *topo = crate::topology::Topology::new();
        }
        if let Ok(mut s) = store.write() {
            s.clear();
        }

        let replay_start = std::time::Instant::now();

        for pkt in &packets {
            let offset = pkt.timestamp.duration_since(first_ts).unwrap_or_default().as_secs_f64();
            let target_elapsed = offset / speedup;
            let actual_elapsed = replay_start.elapsed().as_secs_f64();
            if target_elapsed > actual_elapsed {
                std::thread::sleep(Duration::from_secs_f64(target_elapsed - actual_elapsed));
            }

            ingest_packet(&pkt.data, pkt.timestamp, link_type, &topology, &store);
        }

        eprintln!("replay complete, restarting in 3s...");
        std::thread::sleep(Duration::from_secs(3));
    }
}

pub fn run_capture_live(
    interface: Option<&str>,
    bpf: Option<&str>,
    topology: Arc<RwLock<Topology>>,
    store: Arc<RwLock<PacketStore>>,
) -> Result<()> {
    let mut source = PacketSource::live(interface, 65535, true, bpf, None)?;
    let link_type = source.link_type();

    source.for_each_packet(|pkt: PacketData| {
        ingest_packet(pkt.data, pkt.timestamp, link_type, &topology, &store);
        true
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_pcap() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sample.pcap")
    }

    fn make_store(link_type: LinkType) -> Arc<RwLock<PacketStore>> {
        Arc::new(RwLock::new(PacketStore::new(link_type)))
    }

    fn run_once(
        path: &Path,
        bpf: Option<&str>,
        topology: Arc<RwLock<crate::topology::Topology>>,
        store: Arc<RwLock<PacketStore>>,
    ) -> Result<()> {
        let mut source = PacketSource::from_file(path, bpf)?;
        let link_type = source.link_type();

        source.for_each_packet(|pkt: PacketData| {
            ingest_packet(pkt.data, pkt.timestamp, link_type, &topology, &store);
            true
        })?;

        Ok(())
    }

    #[test]
    fn capture_file_populates_topology() {
        let topo = Arc::new(RwLock::new(crate::topology::Topology::new()));
        let store = make_store(netgrep::protocol::LinkType::Ethernet);
        run_once(&sample_pcap(), None, topo.clone(), store).unwrap();

        let t = topo.read().unwrap();
        assert!(t.total_packets > 0);
        assert!(t.nodes.len() >= 2);
        assert!(!t.edges.is_empty());
    }

    #[test]
    fn capture_file_populates_store() {
        let topo = Arc::new(RwLock::new(crate::topology::Topology::new()));
        let store = make_store(netgrep::protocol::LinkType::Ethernet);
        run_once(&sample_pcap(), None, topo, store.clone()).unwrap();

        let s = store.read().unwrap();
        assert!(s.len() > 0);
    }

    #[test]
    fn capture_file_finds_expected_hosts() {
        let topo = Arc::new(RwLock::new(crate::topology::Topology::new()));
        let store = make_store(netgrep::protocol::LinkType::Ethernet);
        run_once(&sample_pcap(), None, topo.clone(), store).unwrap();

        let t = topo.read().unwrap();
        let ips: Vec<String> = t.nodes.keys().map(|ip| ip.to_string()).collect();
        assert!(ips.contains(&"192.168.1.10".to_string()));
        assert!(ips.contains(&"8.8.8.8".to_string()));
    }

    #[test]
    fn capture_file_classifies_protocols() {
        let topo = Arc::new(RwLock::new(crate::topology::Topology::new()));
        let store = make_store(netgrep::protocol::LinkType::Ethernet);
        run_once(&sample_pcap(), None, topo.clone(), store).unwrap();

        let t = topo.read().unwrap();
        let protos: std::collections::HashSet<String> = t.edges.values().map(|e| e.protocol.clone()).collect();
        assert!(protos.contains("HTTP"));
        assert!(protos.contains("DNS"));
    }

    #[test]
    fn capture_file_nonexistent_errors() {
        let topo = Arc::new(RwLock::new(crate::topology::Topology::new()));
        let store = make_store(netgrep::protocol::LinkType::Ethernet);
        assert!(run_once(Path::new("/nonexistent.pcap"), None, topo, store).is_err());
    }

    #[test]
    fn capture_file_bytes_consistent() {
        let topo = Arc::new(RwLock::new(crate::topology::Topology::new()));
        let store = make_store(netgrep::protocol::LinkType::Ethernet);
        run_once(&sample_pcap(), None, topo.clone(), store).unwrap();

        let t = topo.read().unwrap();
        let total_sent: u64 = t.nodes.values().map(|n| n.bytes_sent).sum();
        assert_eq!(total_sent, t.stats().total_bytes);
    }

    #[test]
    fn capture_file_packet_count_consistent() {
        let topo = Arc::new(RwLock::new(crate::topology::Topology::new()));
        let store = make_store(netgrep::protocol::LinkType::Ethernet);
        run_once(&sample_pcap(), None, topo.clone(), store).unwrap();

        let t = topo.read().unwrap();
        let total_edge_pkts: u64 = t.edges.values().map(|e| e.packets).sum();
        assert_eq!(total_edge_pkts, t.total_packets);
    }

    #[test]
    fn export_pcap_returns_valid_data() {
        let topo = Arc::new(RwLock::new(crate::topology::Topology::new()));
        let store = make_store(netgrep::protocol::LinkType::Ethernet);
        run_once(&sample_pcap(), None, topo, store.clone()).unwrap();

        let s = store.read().unwrap();
        let filter = crate::packet_store::ExportFilter { hosts: vec![], protocols: vec![] };
        let pcap = s.export_pcap(&filter);
        // pcap global header is 24 bytes, should have more
        assert!(pcap.len() > 24);
    }
}
