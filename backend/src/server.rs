use std::sync::{Arc, RwLock};

use anyhow::Result;
use tiny_http::{Header, Response, Server};

use crate::topology::Topology;

pub fn run_server(port: u16, topology: Arc<RwLock<Topology>>) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Failed to bind {}: {}", addr, e))?;

    eprintln!("wiregraph backend listening on http://{}", addr);

    let json_header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
    let cors_header = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap();

    for request in server.incoming_requests() {
        let path = request.url().to_string();
        let (query_since, path_base) = parse_path(&path);

        let response_body = match path_base.as_str() {
            "/api/topology" => {
                let topo = topology.read().unwrap();
                serde_json::to_string(&topo.topology_response()).ok()
            }
            "/api/events" => {
                let topo = topology.read().unwrap();
                let since = query_since.unwrap_or(0.0);
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

fn parse_path(url: &str) -> (Option<f64>, String) {
    match url.split_once('?') {
        Some((path, query)) => {
            let since = query
                .split('&')
                .find_map(|param| {
                    let (k, v) = param.split_once('=')?;
                    if k == "since" { v.parse::<f64>().ok() } else { None }
                });
            (since, path.to_string())
        }
        None => (None, url.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_path_no_query() {
        let (since, path) = parse_path("/api/topology");
        assert_eq!(path, "/api/topology");
        assert!(since.is_none());
    }

    #[test]
    fn parse_path_with_since() {
        let (since, path) = parse_path("/api/events?since=1710500000.5");
        assert_eq!(path, "/api/events");
        assert!((since.unwrap() - 1710500000.5).abs() < 0.001);
    }

    #[test]
    fn parse_path_since_zero() {
        let (since, _) = parse_path("/api/events?since=0");
        assert!((since.unwrap()).abs() < 0.001);
    }

    #[test]
    fn parse_path_since_among_other_params() {
        let (since, path) = parse_path("/api/events?foo=bar&since=42.0&baz=qux");
        assert_eq!(path, "/api/events");
        assert!((since.unwrap() - 42.0).abs() < 0.001);
    }

    #[test]
    fn parse_path_no_since_param() {
        let (since, path) = parse_path("/api/events?foo=bar");
        assert_eq!(path, "/api/events");
        assert!(since.is_none());
    }

    #[test]
    fn parse_path_invalid_since_value() {
        let (since, path) = parse_path("/api/events?since=notanumber");
        assert_eq!(path, "/api/events");
        assert!(since.is_none());
    }

    #[test]
    fn parse_path_empty_since() {
        let (since, _) = parse_path("/api/events?since=");
        assert!(since.is_none());
    }

    #[test]
    fn parse_path_root() {
        let (since, path) = parse_path("/");
        assert_eq!(path, "/");
        assert!(since.is_none());
    }

    #[test]
    fn parse_path_empty_query() {
        let (since, path) = parse_path("/api/stats?");
        assert_eq!(path, "/api/stats");
        assert!(since.is_none());
    }

    #[test]
    fn parse_path_negative_since() {
        let (since, _) = parse_path("/api/events?since=-1.0");
        assert!((since.unwrap() - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn parse_path_large_since() {
        let (since, _) = parse_path("/api/events?since=1710500000000.123");
        assert!(since.is_some());
    }
}
