use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use netgrep::capture::PacketSource;
use netgrep::protocol::{parse_packet, Transport};

use crate::topology::Topology;

fn transport_str(t: Transport, src_port: Option<u16>, dst_port: Option<u16>) -> &'static str {
    match t {
        Transport::Tcp => {
            match dst_port.or(src_port) {
                Some(80) | Some(8080) => "HTTP",
                Some(443) => "TLS",
                Some(22) => "SSH",
                Some(25) | Some(587) | Some(465) => "SMTP",
                _ => "TCP",
            }
        }
        Transport::Udp => {
            match dst_port.or(src_port) {
                Some(53) | Some(5353) => "DNS",
                Some(67) | Some(68) => "DHCP",
                Some(123) => "NTP",
                _ => "UDP",
            }
        }
        Transport::Icmp => "ICMP",
        Transport::Other => "OTHER",
    }
}

pub fn run_capture_file(
    path: &Path,
    bpf: Option<&str>,
    topology: Arc<RwLock<Topology>>,
) -> Result<()> {
    let mut source = PacketSource::from_file(path, bpf)?;
    let link_type = source.link_type();

    source.for_each_packet(|pkt| {
        if let Some(parsed) = parse_packet(pkt.data, link_type) {
            if let (Some(src_ip), Some(dst_ip)) = (parsed.src_ip, parsed.dst_ip) {
                let protocol = transport_str(parsed.transport, parsed.src_port, parsed.dst_port);
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

pub fn run_capture_live(
    interface: Option<&str>,
    bpf: Option<&str>,
    topology: Arc<RwLock<Topology>>,
) -> Result<()> {
    let mut source = PacketSource::live(interface, 65535, true, bpf, None)?;
    let link_type = source.link_type();

    source.for_each_packet(|pkt| {
        if let Some(parsed) = parse_packet(pkt.data, link_type) {
            if let (Some(src_ip), Some(dst_ip)) = (parsed.src_ip, parsed.dst_ip) {
                let protocol = transport_str(parsed.transport, parsed.src_port, parsed.dst_port);
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
