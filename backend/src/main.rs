mod capture;
mod models;
mod server;
mod topology;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

use anyhow::Result;
use clap::Parser;

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

    // Spawn capture thread
    let topo_capture = topology.clone();
    let capture_handle = if let Some(path) = cli.file {
        let bpf = cli.filter.clone();
        thread::spawn(move || {
            if let Err(e) = capture::run_capture_file(
                &path,
                bpf.as_deref(),
                topo_capture,
            ) {
                eprintln!("capture error: {}", e);
            }
            eprintln!("pcap file replay complete");
        })
    } else {
        let iface = cli.interface.clone();
        let bpf = cli.filter.clone();
        thread::spawn(move || {
            if let Err(e) = capture::run_capture_live(
                iface.as_deref(),
                bpf.as_deref(),
                topo_capture,
            ) {
                eprintln!("capture error: {}", e);
            }
        })
    };

    // Run HTTP server on main thread
    server::run_server(cli.port, topology)?;

    let _ = capture_handle.join();
    Ok(())
}
