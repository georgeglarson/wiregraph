use std::net::IpAddr;
use std::time::SystemTime;

use netgrep::capture::pcap_writer::PcapWriter;
use netgrep::protocol::{parse_packet, LinkType, Transport};

use crate::models::classify_protocol;

#[derive(Clone)]
pub struct StoredPacket {
    pub raw: Vec<u8>,
    pub timestamp: SystemTime,
    pub src_ip: Option<IpAddr>,
    pub dst_ip: Option<IpAddr>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub protocol: String,
}

pub struct PacketStore {
    packets: Vec<StoredPacket>,
    link_type: LinkType,
}

impl PacketStore {
    pub fn new(link_type: LinkType) -> Self {
        Self {
            packets: Vec::new(),
            link_type,
        }
    }

    pub fn add(&mut self, raw: &[u8], timestamp: SystemTime) {
        let mut pkt = StoredPacket {
            raw: raw.to_vec(),
            timestamp,
            src_ip: None,
            dst_ip: None,
            src_port: None,
            dst_port: None,
            protocol: "OTHER".to_string(),
        };

        if let Some(parsed) = parse_packet(raw, self.link_type) {
            pkt.src_ip = parsed.src_ip;
            pkt.dst_ip = parsed.dst_ip;
            pkt.src_port = parsed.src_port;
            pkt.dst_port = parsed.dst_port;
            pkt.protocol = classify_protocol(parsed.transport, parsed.src_port, parsed.dst_port).to_string();
        }

        self.packets.push(pkt);
    }

    pub fn clear(&mut self) {
        self.packets.clear();
    }

    pub fn export_pcap(&self, filter: &ExportFilter) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut writer = PcapWriter::new(&mut buf, self.link_type.pcap_link_type())
            .expect("pcap writer");

        for pkt in &self.packets {
            if !filter.matches(pkt) {
                continue;
            }
            let _ = writer.write_packet(&pkt.raw, pkt.timestamp);
        }

        drop(writer);
        buf
    }

    pub fn len(&self) -> usize {
        self.packets.len()
    }
}

pub struct ExportFilter {
    pub hosts: Vec<IpAddr>,
    pub protocols: Vec<String>,
}

impl ExportFilter {
    pub fn matches(&self, pkt: &StoredPacket) -> bool {
        // Empty filter = match all
        let host_match = self.hosts.is_empty() || {
            let src_ok = pkt.src_ip.map_or(false, |ip| self.hosts.contains(&ip));
            let dst_ok = pkt.dst_ip.map_or(false, |ip| self.hosts.contains(&ip));
            src_ok || dst_ok
        };

        let proto_match = self.protocols.is_empty()
            || self.protocols.iter().any(|p| p.eq_ignore_ascii_case(&pkt.protocol));

        host_match && proto_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn make_pkt(src: [u8; 4], dst: [u8; 4], proto: &str) -> StoredPacket {
        StoredPacket {
            raw: vec![0; 64],
            timestamp: SystemTime::now(),
            src_ip: Some(IpAddr::V4(Ipv4Addr::new(src[0], src[1], src[2], src[3]))),
            dst_ip: Some(IpAddr::V4(Ipv4Addr::new(dst[0], dst[1], dst[2], dst[3]))),
            src_port: Some(12345),
            dst_port: Some(80),
            protocol: proto.to_string(),
        }
    }

    #[test]
    fn empty_filter_matches_all() {
        let f = ExportFilter { hosts: vec![], protocols: vec![] };
        assert!(f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
    }

    #[test]
    fn host_filter_matches_src() {
        let f = ExportFilter {
            hosts: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
            protocols: vec![],
        };
        assert!(f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
    }

    #[test]
    fn host_filter_matches_dst() {
        let f = ExportFilter {
            hosts: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2))],
            protocols: vec![],
        };
        assert!(f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
    }

    #[test]
    fn host_filter_no_match() {
        let f = ExportFilter {
            hosts: vec![IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))],
            protocols: vec![],
        };
        assert!(!f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
    }

    #[test]
    fn protocol_filter_matches() {
        let f = ExportFilter { hosts: vec![], protocols: vec!["HTTP".to_string()] };
        assert!(f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
    }

    #[test]
    fn protocol_filter_no_match() {
        let f = ExportFilter { hosts: vec![], protocols: vec!["DNS".to_string()] };
        assert!(!f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
    }

    #[test]
    fn protocol_filter_case_insensitive() {
        let f = ExportFilter { hosts: vec![], protocols: vec!["http".to_string()] };
        assert!(f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
    }

    #[test]
    fn combined_filter_both_must_match() {
        let f = ExportFilter {
            hosts: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
            protocols: vec!["DNS".to_string()],
        };
        // Host matches but protocol doesn't
        assert!(!f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
        // Both match
        assert!(f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "DNS")));
    }

    #[test]
    fn multiple_hosts_filter() {
        let f = ExportFilter {
            hosts: vec![
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
                IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            ],
            protocols: vec![],
        };
        assert!(f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
        assert!(f.matches(&make_pkt([1,2,3,4], [8,8,8,8], "DNS")));
        assert!(!f.matches(&make_pkt([1,2,3,4], [5,6,7,8], "TCP")));
    }

    #[test]
    fn multiple_protocols_filter() {
        let f = ExportFilter {
            hosts: vec![],
            protocols: vec!["HTTP".to_string(), "TLS".to_string()],
        };
        assert!(f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "HTTP")));
        assert!(f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "TLS")));
        assert!(!f.matches(&make_pkt([10,0,0,1], [10,0,0,2], "DNS")));
    }
}
