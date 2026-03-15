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
#[command(name = "wiregraph-backend", about = "Network topology capture backend for wiregraph")]
struct Cli {
    /// Live capture interface
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.interface.is_none() && cli.file.is_none() {
        anyhow::bail!("Must specify either --interface or --file");
    }

    let topology = Arc::new(RwLock::new(Topology::new()));
    // Default to Ethernet; will be overridden once capture starts,
    // but the store needs a link type at construction.
    let store = Arc::new(RwLock::new(PacketStore::new(netgrep::protocol::LinkType::Ethernet)));

    let topo_capture = topology.clone();
    let store_capture = store.clone();
    let capture_handle = if let Some(path) = cli.file {
        let bpf = cli.filter.clone();
        thread::spawn(move || {
            if let Err(e) = capture::run_capture_file(&path, bpf.as_deref(), topo_capture, store_capture) {
                eprintln!("capture error: {}", e);
            }
        })
    } else {
        let iface = cli.interface.clone();
        let bpf = cli.filter.clone();
        thread::spawn(move || {
            if let Err(e) = capture::run_capture_live(iface.as_deref(), bpf.as_deref(), topo_capture, store_capture) {
                eprintln!("capture error: {}", e);
            }
        })
    };

    server::run_server(cli.port, topology, store)?;

    let _ = capture_handle.join();
    Ok(())
}
