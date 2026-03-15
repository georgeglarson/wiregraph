use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use netgrep::capture::pcap_writer::PcapWriter;
use netgrep::protocol::{parse_packet, LinkType};
use serde::Serialize;

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

pub const DEFAULT_MAX_PACKETS: usize = 1_000_000;
pub const DEFAULT_MAX_BYTES: u64 = 500 * 1024 * 1024; // 500 MB

pub struct PacketStore {
    packets: Vec<StoredPacket>,
    link_type: LinkType,
    max_packets: usize,
    max_bytes: u64,
    current_bytes: u64,
    evicted_packets: u64,
    evicted_bytes: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct RetentionInfo {
    pub stored_packets: usize,
    pub stored_bytes: u64,
    pub max_packets: usize,
    pub max_bytes: u64,
    pub evicted_packets: u64,
    pub evicted_bytes: u64,
    pub oldest_timestamp: f64,
    pub newest_timestamp: f64,
    pub window_secs: f64,
    pub utilization_pct: f64,
}

impl PacketStore {
    pub fn new(link_type: LinkType) -> Self {
        Self::with_limits(link_type, DEFAULT_MAX_PACKETS, DEFAULT_MAX_BYTES)
    }

    pub fn with_limits(link_type: LinkType, max_packets: usize, max_bytes: u64) -> Self {
        Self {
            packets: Vec::new(),
            link_type,
            max_packets,
            max_bytes,
            current_bytes: 0,
            evicted_packets: 0,
            evicted_bytes: 0,
        }
    }

    pub fn set_link_type(&mut self, lt: LinkType) {
        self.link_type = lt;
    }

    pub fn add(&mut self, raw: &[u8], timestamp: SystemTime) {
        self.add_with_link_type(raw, timestamp, self.link_type);
    }

    pub fn add_with_link_type(&mut self, raw: &[u8], timestamp: SystemTime, link_type: LinkType) {
        let mut pkt = StoredPacket {
            raw: raw.to_vec(),
            timestamp,
            src_ip: None,
            dst_ip: None,
            src_port: None,
            dst_port: None,
            protocol: "OTHER".to_string(),
        };

        if let Some(parsed) = parse_packet(raw, link_type) {
            pkt.src_ip = parsed.src_ip;
            pkt.dst_ip = parsed.dst_ip;
            pkt.src_port = parsed.src_port;
            pkt.dst_port = parsed.dst_port;
            pkt.protocol = classify_protocol(parsed.transport, parsed.src_port, parsed.dst_port).to_string();
        }

        let pkt_size = pkt.raw.len() as u64;
        self.current_bytes += pkt_size;
        self.packets.push(pkt);

        self.evict();
    }

    fn evict(&mut self) {
        // Batch eviction: drain packets that exceed limits
        let mut to_remove = 0;
        while self.packets.len() - to_remove > self.max_packets
            || (self.current_bytes > self.max_bytes && self.packets.len() - to_remove > 1)
        {
            let size = self.packets[to_remove].raw.len() as u64;
            self.current_bytes -= size;
            self.evicted_packets += 1;
            self.evicted_bytes += size;
            to_remove += 1;
        }
        if to_remove > 0 {
            self.packets.drain(..to_remove);
        }
    }

    pub fn clear(&mut self) {
        self.packets.clear();
        self.current_bytes = 0;
    }

    pub fn retention_info(&self) -> RetentionInfo {
        let oldest = self.packets.first().map(|p| {
            p.timestamp.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64()
        }).unwrap_or(0.0);
        let newest = self.packets.last().map(|p| {
            p.timestamp.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64()
        }).unwrap_or(0.0);
        let window = if oldest > 0.0 && newest > oldest { newest - oldest } else { 0.0 };

        let bytes_util = if self.max_bytes > 0 {
            self.current_bytes as f64 / self.max_bytes as f64 * 100.0
        } else { 0.0 };
        let pkt_util = if self.max_packets > 0 {
            self.packets.len() as f64 / self.max_packets as f64 * 100.0
        } else { 0.0 };

        RetentionInfo {
            stored_packets: self.packets.len(),
            stored_bytes: self.current_bytes,
            max_packets: self.max_packets,
            max_bytes: self.max_bytes,
            evicted_packets: self.evicted_packets,
            evicted_bytes: self.evicted_bytes,
            oldest_timestamp: oldest,
            newest_timestamp: newest,
            window_secs: window,
            utilization_pct: bytes_util.max(pkt_util),
        }
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

    pub fn query_packets(&self, query: &PacketQuery) -> PacketPage {
        let filter = ExportFilter {
            hosts: query.hosts.clone(),
            protocols: query.protocols.clone(),
        };

        let matching: Vec<&StoredPacket> = self.packets.iter()
            .filter(|pkt| filter.matches(pkt))
            .filter(|pkt| {
                query.port.map_or(true, |port| {
                    pkt.src_port == Some(port) || pkt.dst_port == Some(port)
                })
            })
            .collect();

        let total = matching.len();
        let packets: Vec<PacketInfo> = matching.into_iter()
            .skip(query.offset)
            .take(query.limit)
            .map(|pkt| {
                let ts = pkt.timestamp.duration_since(UNIX_EPOCH).unwrap_or_default();
                PacketInfo {
                    timestamp: ts.as_secs_f64(),
                    src_ip: pkt.src_ip.map_or("?".into(), |ip| ip.to_string()),
                    dst_ip: pkt.dst_ip.map_or("?".into(), |ip| ip.to_string()),
                    src_port: pkt.src_port.unwrap_or(0),
                    dst_port: pkt.dst_port.unwrap_or(0),
                    protocol: pkt.protocol.clone(),
                    size: pkt.raw.len(),
                }
            })
            .collect();

        PacketPage {
            packets,
            total,
            offset: query.offset,
            limit: query.limit,
        }
    }

    pub fn conversation(&self, a: IpAddr, b: IpAddr) -> ConversationInfo {
        let mut info = ConversationInfo {
            host_a: a.to_string(),
            host_b: b.to_string(),
            a_to_b_bytes: 0,
            b_to_a_bytes: 0,
            a_to_b_packets: 0,
            b_to_a_packets: 0,
            protocols: HashMap::new(),
            first_seen: 0.0,
            last_seen: 0.0,
            duration: 0.0,
        };

        let mut first: Option<f64> = None;
        let mut last: Option<f64> = None;

        for pkt in &self.packets {
            let src = match pkt.src_ip { Some(ip) => ip, None => continue };
            let dst = match pkt.dst_ip { Some(ip) => ip, None => continue };

            let is_a_to_b = src == a && dst == b;
            let is_b_to_a = src == b && dst == a;

            if !is_a_to_b && !is_b_to_a {
                continue;
            }

            let ts = pkt.timestamp.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();
            first = Some(first.map_or(ts, |f: f64| f.min(ts)));
            last = Some(last.map_or(ts, |l: f64| l.max(ts)));

            let size = pkt.raw.len() as u64;
            if is_a_to_b {
                info.a_to_b_bytes += size;
                info.a_to_b_packets += 1;
            } else {
                info.b_to_a_bytes += size;
                info.b_to_a_packets += 1;
            }

            *info.protocols.entry(pkt.protocol.clone()).or_insert(0) += size;
        }

        if let (Some(f), Some(l)) = (first, last) {
            info.first_seen = f;
            info.last_seen = l;
            info.duration = l - f;
        }

        info
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

// --- Packet query types ---

pub struct PacketQuery {
    pub hosts: Vec<IpAddr>,
    pub protocols: Vec<String>,
    pub port: Option<u16>,
    pub limit: usize,
    pub offset: usize,
}

impl Default for PacketQuery {
    fn default() -> Self {
        Self {
            hosts: vec![],
            protocols: vec![],
            port: None,
            limit: 100,
            offset: 0,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct PacketInfo {
    pub timestamp: f64,
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: String,
    pub size: usize,
}

#[derive(Serialize)]
pub struct PacketPage {
    pub packets: Vec<PacketInfo>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Serialize, Debug)]
pub struct ConversationInfo {
    pub host_a: String,
    pub host_b: String,
    pub a_to_b_bytes: u64,
    pub b_to_a_bytes: u64,
    pub a_to_b_packets: u64,
    pub b_to_a_packets: u64,
    pub protocols: HashMap<String, u64>,
    pub first_seen: f64,
    pub last_seen: f64,
    pub duration: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn make_pkt(src: [u8; 4], dst: [u8; 4], proto: &str) -> StoredPacket {
        make_pkt_ports(src, dst, proto, 12345, 80)
    }

    fn make_pkt_ports(src: [u8; 4], dst: [u8; 4], proto: &str, sport: u16, dport: u16) -> StoredPacket {
        StoredPacket {
            raw: vec![0; 64],
            timestamp: SystemTime::now(),
            src_ip: Some(IpAddr::V4(Ipv4Addr::new(src[0], src[1], src[2], src[3]))),
            dst_ip: Some(IpAddr::V4(Ipv4Addr::new(dst[0], dst[1], dst[2], dst[3]))),
            src_port: Some(sport),
            dst_port: Some(dport),
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

    // --- query_packets tests ---

    fn make_store_with_packets() -> PacketStore {
        let mut store = PacketStore::new(LinkType::Ethernet);
        store.packets.push(make_pkt_ports([10,0,0,1], [10,0,0,2], "HTTP", 54321, 80));
        store.packets.push(make_pkt_ports([10,0,0,1], [8,8,8,8], "DNS", 54322, 53));
        store.packets.push(make_pkt_ports([10,0,0,2], [10,0,0,1], "TLS", 54323, 443));
        store.packets.push(make_pkt_ports([10,0,0,1], [10,0,0,2], "HTTP", 54324, 80));
        store.packets.push(make_pkt_ports([8,8,8,8], [10,0,0,1], "DNS", 53, 54322));
        store
    }

    #[test]
    fn query_packets_no_filter() {
        let store = make_store_with_packets();
        let page = store.query_packets(&PacketQuery::default());
        assert_eq!(page.total, 5);
        assert_eq!(page.packets.len(), 5);
    }

    #[test]
    fn query_packets_host_filter() {
        let store = make_store_with_packets();
        let page = store.query_packets(&PacketQuery {
            hosts: vec![IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))],
            ..Default::default()
        });
        assert_eq!(page.total, 2);
    }

    #[test]
    fn query_packets_protocol_filter() {
        let store = make_store_with_packets();
        let page = store.query_packets(&PacketQuery {
            protocols: vec!["HTTP".to_string()],
            ..Default::default()
        });
        assert_eq!(page.total, 2);
    }

    #[test]
    fn query_packets_port_filter() {
        let store = make_store_with_packets();
        let page = store.query_packets(&PacketQuery {
            port: Some(443),
            ..Default::default()
        });
        assert_eq!(page.total, 1);
        assert_eq!(page.packets[0].protocol, "TLS");
    }

    #[test]
    fn query_packets_pagination() {
        let store = make_store_with_packets();
        let page = store.query_packets(&PacketQuery {
            limit: 2,
            offset: 0,
            ..Default::default()
        });
        assert_eq!(page.total, 5);
        assert_eq!(page.packets.len(), 2);
        assert_eq!(page.offset, 0);
        assert_eq!(page.limit, 2);

        let page2 = store.query_packets(&PacketQuery {
            limit: 2,
            offset: 2,
            ..Default::default()
        });
        assert_eq!(page2.total, 5);
        assert_eq!(page2.packets.len(), 2);
        assert_eq!(page2.offset, 2);
    }

    #[test]
    fn query_packets_combined_filters() {
        let store = make_store_with_packets();
        let page = store.query_packets(&PacketQuery {
            hosts: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))],
            port: Some(80),
            ..Default::default()
        });
        assert_eq!(page.total, 2);
        for p in &page.packets {
            assert_eq!(p.protocol, "HTTP");
        }
    }

    // --- conversation tests ---

    #[test]
    fn conversation_basic() {
        let store = make_store_with_packets();
        let a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        let conv = store.conversation(a, b);
        assert_eq!(conv.host_a, "10.0.0.1");
        assert_eq!(conv.host_b, "10.0.0.2");
        // 10.0.0.1 -> 10.0.0.2: 2 HTTP packets (64 bytes each)
        assert_eq!(conv.a_to_b_packets, 2);
        assert_eq!(conv.a_to_b_bytes, 128);
        // 10.0.0.2 -> 10.0.0.1: 1 TLS packet
        assert_eq!(conv.b_to_a_packets, 1);
        assert_eq!(conv.b_to_a_bytes, 64);
        assert!(conv.protocols.contains_key("HTTP"));
        assert!(conv.protocols.contains_key("TLS"));
    }

    // --- eviction tests ---

    #[test]
    fn eviction_by_packet_count() {
        let mut store = PacketStore::with_limits(LinkType::Ethernet, 3, u64::MAX);
        for i in 0..5u8 {
            store.packets.push(make_pkt_ports([10,0,0,i], [10,0,0,2], "TCP", 1000 + i as u16, 80));
            store.current_bytes += 64;
        }
        store.evict();
        assert_eq!(store.packets.len(), 3);
        assert_eq!(store.evicted_packets, 2);
        // Oldest packets (i=0,1) should be gone, remaining are i=2,3,4
        assert_eq!(store.packets[0].src_port, Some(1002));
    }

    #[test]
    fn eviction_by_byte_limit() {
        // 64 bytes per packet, limit to 192 bytes = 3 packets
        let mut store = PacketStore::with_limits(LinkType::Ethernet, usize::MAX, 192);
        for _ in 0..5 {
            store.packets.push(make_pkt([10,0,0,1], [10,0,0,2], "TCP"));
            store.current_bytes += 64;
        }
        store.evict();
        assert_eq!(store.packets.len(), 3);
        assert_eq!(store.current_bytes, 192);
        assert_eq!(store.evicted_bytes, 128);
    }

    #[test]
    fn add_evicts_automatically() {
        let mut store = PacketStore::with_limits(LinkType::Ethernet, 3, u64::MAX);
        for _ in 0..5 {
            store.add(&[0u8; 64], SystemTime::now());
        }
        assert_eq!(store.packets.len(), 3);
        assert_eq!(store.evicted_packets, 2);
    }

    #[test]
    fn retention_info_reports_correctly() {
        let mut store = PacketStore::with_limits(LinkType::Ethernet, 100, 10000);
        for _ in 0..5 {
            store.add(&[0u8; 64], SystemTime::now());
        }
        let ri = store.retention_info();
        assert_eq!(ri.stored_packets, 5);
        assert_eq!(ri.max_packets, 100);
        assert_eq!(ri.max_bytes, 10000);
        assert_eq!(ri.evicted_packets, 0);
    }

    #[test]
    fn conversation_no_traffic() {
        let store = make_store_with_packets();
        let a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
        let conv = store.conversation(a, b);
        assert_eq!(conv.a_to_b_packets, 0);
        assert_eq!(conv.b_to_a_packets, 0);
        assert_eq!(conv.duration, 0.0);
    }
}
