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
