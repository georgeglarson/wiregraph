mod capture;
mod models;
mod packet_store;
mod server;
mod topology;
mod web_ui;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

use anyhow::Result;
use clap::Parser;

use packet_store::PacketStore;
use topology::Topology;

#[derive(Parser)]
#[command(name = "wiregraph-backend", about = "Network traffic capture backend for wiregraph")]
struct Cli {
    /// Live capture interfaces (comma-separated, or omit for interactive selection)
    #[arg(short, long)]
    interface: Option<String>,

    /// Load pcap file
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// HTTP port
    #[arg(short, long, default_value = "9877")]
    port: u16,

    /// BPF filter expression
    #[arg(long)]
    filter: Option<String>,

    /// List available interfaces and exit
    #[arg(short = 'L', long)]
    list_interfaces: bool,

    /// Max packets to retain (rolling window). Default: 1000000
    #[arg(long, default_value = "1000000")]
    max_packets: usize,

    /// Max bytes to retain (rolling window). Default: 500MB. Supports K/M/G suffix
    #[arg(long, default_value = "500M")]
    max_bytes: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_interfaces {
        let ifaces = capture::list_interfaces()?;
        if ifaces.is_empty() {
            eprintln!("no interfaces found (try running with sudo)");
        } else {
            for (i, iface) in ifaces.iter().enumerate() {
                let addrs = if iface.addresses.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", iface.addresses.join(", "))
                };
                let desc = if iface.description.is_empty() {
                    String::new()
                } else {
                    format!(" — {}", iface.description)
                };
                eprintln!("  {:>2}) {}{}{}", i + 1, iface.name, addrs, desc);
            }
        }
        return Ok(());
    }

    // Resolve interfaces for live capture
    let interfaces: Vec<String> = if let Some(ref iface_arg) = cli.interface {
        iface_arg.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    } else if cli.file.is_none() {
        // Interactive selection
        let ifaces = capture::list_interfaces()?;
        if ifaces.is_empty() {
            anyhow::bail!("No interfaces found (try running with sudo)");
        }
        eprintln!("Available interfaces:");
        for (i, iface) in ifaces.iter().enumerate() {
            let addrs = if iface.addresses.is_empty() {
                String::new()
            } else {
                format!(" [{}]", iface.addresses.join(", "))
            };
            let desc = if iface.description.is_empty() {
                String::new()
            } else {
                format!(" — {}", iface.description)
            };
            eprintln!("  {:>2}) {}{}{}", i + 1, iface.name, addrs, desc);
        }
        eprint!("\nSelect interfaces (numbers or names, comma-separated): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let selected: Vec<String> = input.trim().split(',').filter_map(|s| {
            let s = s.trim();
            if s.is_empty() { return None; }
            // Try as number first
            if let Ok(n) = s.parse::<usize>() {
                if n >= 1 && n <= ifaces.len() {
                    return Some(ifaces[n - 1].name.clone());
                }
            }
            // Try as name
            if ifaces.iter().any(|i| i.name == s) {
                Some(s.to_string())
            } else {
                eprintln!("warning: '{}' not found, skipping", s);
                None
            }
        }).collect();
        if selected.is_empty() {
            anyhow::bail!("No valid interfaces selected");
        }
        selected
    } else {
        vec![]
    };

    if interfaces.is_empty() && cli.file.is_none() {
        anyhow::bail!("Must specify --interface, --file, or select interactively");
    }

    let max_bytes = parse_byte_size(&cli.max_bytes)
        .ok_or_else(|| anyhow::anyhow!("Invalid --max-bytes value: '{}'. Use e.g. 500M, 1G, 2048K", cli.max_bytes))?;
    eprintln!("retention: max {} packets, {} buffer",
        cli.max_packets, format_bytes(max_bytes));

    let topology = Arc::new(RwLock::new(Topology::new()));
    let store = Arc::new(RwLock::new(
        PacketStore::with_limits(netgrep::protocol::LinkType::Ethernet, cli.max_packets, max_bytes)
    ));

    let capture_handles: Vec<_> = if let Some(path) = cli.file {
        let topo = topology.clone();
        let st = store.clone();
        let bpf = cli.filter.clone();
        vec![thread::spawn(move || {
            if let Err(e) = capture::run_capture_file(&path, bpf.as_deref(), topo, st) {
                eprintln!("capture error: {}", e);
            }
        })]
    } else {
        interfaces.iter().map(|iface| {
            let topo = topology.clone();
            let st = store.clone();
            let bpf = cli.filter.clone();
            let iface = iface.clone();
            thread::spawn(move || {
                eprintln!("capturing on {}", iface);
                if let Err(e) = capture::run_capture_live(Some(&iface), bpf.as_deref(), topo, st) {
                    eprintln!("capture error on {}: {}", iface, e);
                }
            })
        }).collect()
    };

    // Store interface list for the API
    let iface_list = capture::list_interfaces().unwrap_or_default();

    server::run_server(cli.port, topology, store, iface_list)?;

    for h in capture_handles {
        let _ = h.join();
    }
    Ok(())
}

fn parse_byte_size(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() { return None; }
    let (num_part, multiplier) = match s.as_bytes().last()? {
        b'K' | b'k' => (&s[..s.len()-1], 1024u64),
        b'M' | b'm' => (&s[..s.len()-1], 1024 * 1024),
        b'G' | b'g' => (&s[..s.len()-1], 1024 * 1024 * 1024),
        b'B' | b'b' => {
            // Handle KB, MB, GB
            if s.len() >= 2 {
                match s.as_bytes()[s.len()-2] {
                    b'K' | b'k' => (&s[..s.len()-2], 1024u64),
                    b'M' | b'm' => (&s[..s.len()-2], 1024 * 1024),
                    b'G' | b'g' => (&s[..s.len()-2], 1024 * 1024 * 1024),
                    _ => (s, 1u64),
                }
            } else {
                (s, 1u64)
            }
        }
        _ => (s, 1u64),
    };
    let num: u64 = num_part.trim().parse().ok()?;
    Some(num * multiplier)
}

fn format_bytes(b: u64) -> String {
    if b >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", b as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if b >= 1024 * 1024 {
        format!("{:.0} MB", b as f64 / (1024.0 * 1024.0))
    } else if b >= 1024 {
        format!("{:.0} KB", b as f64 / 1024.0)
    } else {
        format!("{} B", b)
    }
}
