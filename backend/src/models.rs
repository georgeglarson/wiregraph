use netgrep::protocol::Transport;
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

pub fn classify_protocol(t: Transport, src_port: Option<u16>, dst_port: Option<u16>) -> &'static str {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    // -- subnet_of --

    #[test]
    fn subnet_of_ipv4_class_a() {
        assert_eq!(subnet_of(IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3))), "10.1.2.0/24");
    }

    #[test]
    fn subnet_of_ipv4_class_c() {
        assert_eq!(subnet_of(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))), "192.168.1.0/24");
    }

    #[test]
    fn subnet_of_ipv4_zeros() {
        assert_eq!(subnet_of(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))), "0.0.0.0/24");
    }

    #[test]
    fn subnet_of_ipv4_broadcast() {
        assert_eq!(subnet_of(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255))), "255.255.255.0/24");
    }

    #[test]
    fn subnet_of_ipv6() {
        let ip = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0xabcd, 0x0012, 0, 0, 0, 1));
        assert_eq!(subnet_of(ip), "2001:db8:abcd:12::/64");
    }

    #[test]
    fn subnet_of_ipv6_loopback() {
        assert_eq!(subnet_of(IpAddr::V6(Ipv6Addr::LOCALHOST)), "0:0:0:0::/64");
    }

    // -- is_local_ip --

    #[test]
    fn local_10_private() {
        assert!(is_local_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    }

    #[test]
    fn local_172_16_private() {
        assert!(is_local_ip(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
    }

    #[test]
    fn local_192_168_private() {
        assert!(is_local_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
    }

    #[test]
    fn local_loopback() {
        assert!(is_local_ip(IpAddr::V4(Ipv4Addr::LOCALHOST)));
    }

    #[test]
    fn local_link_local() {
        assert!(is_local_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))));
    }

    #[test]
    fn not_local_public() {
        assert!(!is_local_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }

    #[test]
    fn not_local_public_93() {
        assert!(!is_local_ip(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))));
    }

    #[test]
    fn not_local_172_32() {
        // 172.32.x.x is NOT private (only 172.16-31.x.x)
        assert!(!is_local_ip(IpAddr::V4(Ipv4Addr::new(172, 32, 0, 1))));
    }

    #[test]
    fn local_ipv6_loopback() {
        assert!(is_local_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn local_ipv6_ula() {
        // fc00::/7
        let ip = IpAddr::V6(Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 1));
        assert!(is_local_ip(ip));
    }

    #[test]
    fn local_ipv6_link_local() {
        // fe80::/10
        let ip = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
        assert!(is_local_ip(ip));
    }

    #[test]
    fn not_local_ipv6_global() {
        let ip = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
        assert!(!is_local_ip(ip));
    }

    // -- Node::new --

    #[test]
    fn node_new_defaults() {
        let node = Node::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert_eq!(node.ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert_eq!(node.subnet, "10.0.0.0/24");
        assert_eq!(node.bytes_sent, 0);
        assert_eq!(node.bytes_recv, 0);
        assert_eq!(node.packet_count, 0);
        assert!(node.protocols.is_empty());
        assert!(node.is_local);
        assert_eq!(node.last_seen, 0.0);
    }

    #[test]
    fn node_new_public_ip() {
        let node = Node::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(!node.is_local);
        assert_eq!(node.subnet, "8.8.8.0/24");
    }

    // -- Edge::new --

    #[test]
    fn edge_new_defaults() {
        let src = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let dst = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        let edge = Edge::new(src, dst, 80, "HTTP");
        assert_eq!(edge.source, src);
        assert_eq!(edge.target, dst);
        assert_eq!(edge.dst_port, 80);
        assert_eq!(edge.protocol, "HTTP");
        assert_eq!(edge.bytes, 0);
        assert_eq!(edge.packets, 0);
        assert!(edge.active);
        assert_eq!(edge.last_seen, 0.0);
    }

    // -- classify_protocol --

    #[test]
    fn classify_tcp_http_80() {
        assert_eq!(classify_protocol(Transport::Tcp, None, Some(80)), "HTTP");
    }

    #[test]
    fn classify_tcp_http_8080() {
        assert_eq!(classify_protocol(Transport::Tcp, None, Some(8080)), "HTTP");
    }

    #[test]
    fn classify_tcp_tls() {
        assert_eq!(classify_protocol(Transport::Tcp, None, Some(443)), "TLS");
    }

    #[test]
    fn classify_tcp_ssh() {
        assert_eq!(classify_protocol(Transport::Tcp, None, Some(22)), "SSH");
    }

    #[test]
    fn classify_tcp_smtp_25() {
        assert_eq!(classify_protocol(Transport::Tcp, None, Some(25)), "SMTP");
    }

    #[test]
    fn classify_tcp_smtp_587() {
        assert_eq!(classify_protocol(Transport::Tcp, None, Some(587)), "SMTP");
    }

    #[test]
    fn classify_tcp_smtp_465() {
        assert_eq!(classify_protocol(Transport::Tcp, None, Some(465)), "SMTP");
    }

    #[test]
    fn classify_tcp_unknown_port() {
        assert_eq!(classify_protocol(Transport::Tcp, None, Some(12345)), "TCP");
    }

    #[test]
    fn classify_tcp_no_ports() {
        assert_eq!(classify_protocol(Transport::Tcp, None, None), "TCP");
    }

    #[test]
    fn classify_tcp_falls_back_to_src_port() {
        // dst_port is None, should check src_port
        assert_eq!(classify_protocol(Transport::Tcp, Some(80), None), "HTTP");
    }

    #[test]
    fn classify_udp_dns_53() {
        assert_eq!(classify_protocol(Transport::Udp, None, Some(53)), "DNS");
    }

    #[test]
    fn classify_udp_mdns_5353() {
        assert_eq!(classify_protocol(Transport::Udp, None, Some(5353)), "DNS");
    }

    #[test]
    fn classify_udp_dhcp_67() {
        assert_eq!(classify_protocol(Transport::Udp, None, Some(67)), "DHCP");
    }

    #[test]
    fn classify_udp_dhcp_68() {
        assert_eq!(classify_protocol(Transport::Udp, None, Some(68)), "DHCP");
    }

    #[test]
    fn classify_udp_ntp() {
        assert_eq!(classify_protocol(Transport::Udp, None, Some(123)), "NTP");
    }

    #[test]
    fn classify_udp_unknown() {
        assert_eq!(classify_protocol(Transport::Udp, None, Some(9999)), "UDP");
    }

    #[test]
    fn classify_udp_no_ports() {
        assert_eq!(classify_protocol(Transport::Udp, None, None), "UDP");
    }

    #[test]
    fn classify_udp_falls_back_to_src_port() {
        assert_eq!(classify_protocol(Transport::Udp, Some(53), None), "DNS");
    }

    #[test]
    fn classify_icmp() {
        assert_eq!(classify_protocol(Transport::Icmp, None, None), "ICMP");
    }

    #[test]
    fn classify_icmp_ignores_ports() {
        assert_eq!(classify_protocol(Transport::Icmp, Some(80), Some(443)), "ICMP");
    }

    #[test]
    fn classify_other() {
        assert_eq!(classify_protocol(Transport::Other, None, None), "OTHER");
    }

    // -- EdgeKey equality --

    #[test]
    fn edge_key_same_fields_equal() {
        let a = EdgeKey {
            source: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            target: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            dst_port: 80,
        };
        let b = EdgeKey {
            source: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            target: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            dst_port: 80,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn edge_key_different_port_not_equal() {
        let a = EdgeKey {
            source: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            target: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            dst_port: 80,
        };
        let b = EdgeKey {
            source: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            target: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            dst_port: 443,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn edge_key_reversed_direction_not_equal() {
        // EdgeKey is directional — A→B ≠ B→A
        let a = EdgeKey {
            source: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            target: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            dst_port: 80,
        };
        let b = EdgeKey {
            source: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            target: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            dst_port: 80,
        };
        assert_ne!(a, b);
    }

    // -- Serialization round-trip --

    #[test]
    fn node_serializes_to_json() {
        let node = Node::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"ip\":\"10.0.0.1\""));
        assert!(json.contains("\"subnet\":\"10.0.0.0/24\""));
        assert!(json.contains("\"is_local\":true"));
    }

    #[test]
    fn edge_serializes_to_json() {
        let src = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let dst = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        let edge = Edge::new(src, dst, 53, "DNS");
        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("\"protocol\":\"DNS\""));
        assert!(json.contains("\"dst_port\":53"));
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn stats_serializes_to_json() {
        let stats = Stats {
            total_packets: 100,
            total_bytes: 50000,
            host_count: 5,
            edge_count: 8,
            packets_per_second: 33.3,
            capture_duration: 3.0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_packets\":100"));
        assert!(json.contains("\"host_count\":5"));
    }

    #[test]
    fn topology_response_serializes_to_json() {
        let resp = TopologyResponse {
            nodes: vec![Node::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)))],
            edges: vec![],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"nodes\":["));
        assert!(json.contains("\"edges\":[]"));
    }
}
