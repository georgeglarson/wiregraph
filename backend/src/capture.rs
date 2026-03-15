use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use anyhow::Result;
use netgrep::capture::{PacketData, PacketSource};
use netgrep::protocol::parse_packet;

use crate::models::classify_protocol;
use crate::topology::Topology;

struct CapturedPacket {
    data: Vec<u8>,
    timestamp: SystemTime,
}

fn ingest_packet(pkt: &CapturedPacket, link_type: netgrep::protocol::LinkType, topology: &Arc<RwLock<Topology>>) {
    if let Some(parsed) = parse_packet(&pkt.data, link_type) {
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
                    None, // use wall clock so events stay "live"
                );
            }
        }
    }
}

pub fn run_capture_file(
    path: &Path,
    bpf: Option<&str>,
    topology: Arc<RwLock<Topology>>,
) -> Result<()> {
    // Read all packets into memory first
    let mut source = PacketSource::from_file(path, bpf)?;
    let link_type = source.link_type();
    let mut packets: Vec<CapturedPacket> = Vec::new();

    source.for_each_packet(|pkt: PacketData| {
        packets.push(CapturedPacket {
            data: pkt.data.to_vec(),
            timestamp: pkt.timestamp,
        });
        true
    })?;

    if packets.is_empty() {
        eprintln!("no packets in pcap file");
        return Ok(());
    }

    // Compute time span of the capture
    let first_ts = packets[0].timestamp;
    let last_ts = packets[packets.len() - 1].timestamp;
    let capture_span = last_ts
        .duration_since(first_ts)
        .unwrap_or(Duration::from_secs(1));

    // Replay duration: stretch short captures to at least 8 seconds
    let replay_secs = 8.0f64;
    let speedup = if capture_span.as_secs_f64() > 0.001 {
        capture_span.as_secs_f64() / replay_secs
    } else {
        0.001
    };

    eprintln!(
        "replaying {} packets over {:.0}s (original span: {:.2}s)",
        packets.len(),
        replay_secs,
        capture_span.as_secs_f64()
    );

    // Loop forever, replaying the pcap
    loop {
        // Clear topology for fresh replay
        if let Ok(mut topo) = topology.write() {
            *topo = Topology::new();
        }

        let replay_start = std::time::Instant::now();

        for pkt in &packets {
            // How far into the original capture is this packet?
            let offset = pkt.timestamp
                .duration_since(first_ts)
                .unwrap_or_default()
                .as_secs_f64();

            // Scale to replay time
            let target_elapsed = offset / speedup;
            let actual_elapsed = replay_start.elapsed().as_secs_f64();

            if target_elapsed > actual_elapsed {
                let wait = Duration::from_secs_f64(target_elapsed - actual_elapsed);
                std::thread::sleep(wait);
            }

            ingest_packet(pkt, link_type, &topology);
        }

        eprintln!("replay complete, restarting in 3s...");
        std::thread::sleep(Duration::from_secs(3));
    }
}

pub fn run_capture_live(
    interface: Option<&str>,
    bpf: Option<&str>,
    topology: Arc<RwLock<Topology>>,
) -> Result<()> {
    let mut source = PacketSource::live(interface, 65535, true, bpf, None)?;
    let link_type = source.link_type();

    source.for_each_packet(|pkt: PacketData| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_pcap() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sample.pcap")
    }

    #[test]
    fn capture_file_populates_topology() {
        // Use run_capture_file_once for tests (non-looping)
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file_once(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        assert!(t.total_packets > 0, "should have captured packets");
        assert!(t.nodes.len() >= 2, "should have at least 2 hosts");
        assert!(!t.edges.is_empty(), "should have at least 1 edge");
    }

    #[test]
    fn capture_file_finds_expected_hosts() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file_once(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        let ips: Vec<String> = t.nodes.keys().map(|ip| ip.to_string()).collect();

        assert!(ips.contains(&"192.168.1.10".to_string()), "missing local host");
        assert!(ips.contains(&"8.8.8.8".to_string()), "missing DNS server");
    }

    #[test]
    fn capture_file_classifies_protocols() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file_once(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        let all_protocols: std::collections::HashSet<String> = t.edges.values()
            .map(|e| e.protocol.clone())
            .collect();

        assert!(all_protocols.contains("HTTP"), "missing HTTP edges");
        assert!(all_protocols.contains("DNS"), "missing DNS edges");
    }

    #[test]
    fn capture_file_events_populated() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file_once(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        assert!(!t.events.is_empty(), "events ring buffer should have entries");
    }

    #[test]
    fn capture_file_nonexistent_errors() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        let result = run_capture_file_once(Path::new("/nonexistent.pcap"), None, topo);
        assert!(result.is_err());
    }

    #[test]
    fn capture_file_bytes_consistent() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file_once(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        let stats = t.stats();

        let total_sent: u64 = t.nodes.values().map(|n| n.bytes_sent).sum();
        assert_eq!(total_sent, stats.total_bytes, "node bytes_sent should sum to total");
    }

    #[test]
    fn capture_file_packet_count_consistent() {
        let topo = Arc::new(RwLock::new(Topology::new()));
        run_capture_file_once(&sample_pcap(), None, topo.clone()).unwrap();

        let t = topo.read().unwrap();
        let total_edge_packets: u64 = t.edges.values().map(|e| e.packets).sum();
        assert_eq!(total_edge_packets, t.total_packets, "edge packets should sum to total");
    }
}

// Non-looping version for tests
#[cfg(test)]
fn run_capture_file_once(
    path: &Path,
    bpf: Option<&str>,
    topology: Arc<RwLock<Topology>>,
) -> Result<()> {
    let mut source = PacketSource::from_file(path, bpf)?;
    let link_type = source.link_type();

    source.for_each_packet(|pkt: PacketData| {
        if let Some(parsed) = parse_packet(pkt.data, link_type) {
            if let (Some(src_ip), Some(dst_ip)) = (parsed.src_ip, parsed.dst_ip) {
                let protocol = classify_protocol(parsed.transport, parsed.src_port, parsed.dst_port);
                let bytes = pkt.data.len() as u64;

                if let Ok(mut topo) = topology.write() {
                    topo.ingest(
                        src_ip, dst_ip,
                        parsed.src_port, parsed.dst_port,
                        protocol, bytes, Some(pkt.timestamp),
                    );
                }
            }
        }
        true
    })?;

    Ok(())
}
