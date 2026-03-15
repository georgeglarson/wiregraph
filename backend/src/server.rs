use std::sync::{Arc, RwLock};

use anyhow::Result;
use tiny_http::{Header, Response, Server};

use crate::packet_store::{ExportFilter, PacketStore};
use crate::topology::Topology;
use crate::web_ui;

pub fn run_server(
    port: u16,
    topology: Arc<RwLock<Topology>>,
    store: Arc<RwLock<PacketStore>>,
) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Failed to bind {}: {}", addr, e))?;

    eprintln!("wiregraph backend listening on http://{}", addr);

    let json_header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
    let cors_header = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap();

    for request in server.incoming_requests() {
        let path = request.url().to_string();
        let (params, path_base) = parse_path(&path);

        // Serve web UI
        if path_base == "/" || path_base == "/index.html" {
            let html_header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();
            let _ = request.respond(Response::from_string(web_ui::INDEX_HTML).with_header(html_header));
            continue;
        }

        // Pcap export
        if path_base == "/api/export" {
            let filter = build_export_filter(&params);
            let s = store.read().unwrap();
            let pcap_data = s.export_pcap(&filter);

            let pcap_header = Header::from_bytes(&b"Content-Type"[..], &b"application/vnd.tcpdump.pcap"[..]).unwrap();
            let disp_header = Header::from_bytes(
                &b"Content-Disposition"[..],
                &b"attachment; filename=\"wiregraph-export.pcap\""[..],
            ).unwrap();
            let _ = request.respond(
                Response::from_data(pcap_data)
                    .with_header(pcap_header)
                    .with_header(disp_header)
                    .with_header(cors_header.clone()),
            );
            continue;
        }

        // JSON API
        let response_body = match path_base.as_str() {
            "/api/topology" => {
                let topo = topology.read().unwrap();
                serde_json::to_string(&topo.topology_response()).ok()
            }
            "/api/events" => {
                let topo = topology.read().unwrap();
                let since = params.get("since").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
                serde_json::to_string(&topo.events_since(since)).ok()
            }
            "/api/stats" => {
                let topo = topology.read().unwrap();
                serde_json::to_string(&topo.stats()).ok()
            }
            _ => None,
        };

        let response = match response_body {
            Some(body) => Response::from_string(body)
                .with_header(json_header.clone())
                .with_header(cors_header.clone()),
            None => Response::from_string(r#"{"error":"not found"}"#)
                .with_status_code(404)
                .with_header(json_header.clone())
                .with_header(cors_header.clone()),
        };

        let _ = request.respond(response);
    }

    Ok(())
}

fn parse_path(url: &str) -> (std::collections::HashMap<String, String>, String) {
    match url.split_once('?') {
        Some((path, query)) => {
            let params: std::collections::HashMap<String, String> = query
                .split('&')
                .filter_map(|param| {
                    let (k, v) = param.split_once('=')?;
                    Some((k.to_string(), v.to_string()))
                })
                .collect();
            (params, path.to_string())
        }
        None => (std::collections::HashMap::new(), url.to_string()),
    }
}

fn build_export_filter(params: &std::collections::HashMap<String, String>) -> ExportFilter {
    let hosts = params.get("hosts").map(|h| {
        h.split(',')
            .filter_map(|ip| ip.parse().ok())
            .collect()
    }).unwrap_or_default();

    let protocols = params.get("protocols").map(|p| {
        p.split(',')
            .map(|s| s.to_string())
            .collect()
    }).unwrap_or_default();

    ExportFilter { hosts, protocols }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_path_no_query() {
        let (params, path) = parse_path("/api/topology");
        assert_eq!(path, "/api/topology");
        assert!(params.is_empty());
    }

    #[test]
    fn parse_path_with_params() {
        let (params, path) = parse_path("/api/events?since=1710500000.5");
        assert_eq!(path, "/api/events");
        assert_eq!(params.get("since").unwrap(), "1710500000.5");
    }

    #[test]
    fn parse_path_multiple_params() {
        let (params, path) = parse_path("/api/export?hosts=10.0.0.1,8.8.8.8&protocols=HTTP,DNS");
        assert_eq!(path, "/api/export");
        assert_eq!(params.get("hosts").unwrap(), "10.0.0.1,8.8.8.8");
        assert_eq!(params.get("protocols").unwrap(), "HTTP,DNS");
    }

    #[test]
    fn build_filter_empty() {
        let params = std::collections::HashMap::new();
        let f = build_export_filter(&params);
        assert!(f.hosts.is_empty());
        assert!(f.protocols.is_empty());
    }

    #[test]
    fn build_filter_hosts() {
        let mut params = std::collections::HashMap::new();
        params.insert("hosts".to_string(), "10.0.0.1,8.8.8.8".to_string());
        let f = build_export_filter(&params);
        assert_eq!(f.hosts.len(), 2);
    }

    #[test]
    fn build_filter_protocols() {
        let mut params = std::collections::HashMap::new();
        params.insert("protocols".to_string(), "HTTP,DNS,TLS".to_string());
        let f = build_export_filter(&params);
        assert_eq!(f.protocols.len(), 3);
    }

    #[test]
    fn build_filter_invalid_ip_skipped() {
        let mut params = std::collections::HashMap::new();
        params.insert("hosts".to_string(), "10.0.0.1,notanip,8.8.8.8".to_string());
        let f = build_export_filter(&params);
        assert_eq!(f.hosts.len(), 2);
    }
}
