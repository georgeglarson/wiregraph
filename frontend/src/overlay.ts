import { Node, Stats, PROTOCOL_COLORS } from "./types";

export class Overlay {
  private stats: Stats | null = null;
  private selectedNode: Node | null = null;
  private width: number;
  private height: number;
  private lastLog = 0;
  private startupPrinted = false;

  constructor(width: number, height: number) {
    this.width = width;
    this.height = height;
  }

  setStats(stats: Stats): void {
    this.stats = stats;
  }

  setSelectedNode(node: Node | null): void {
    this.selectedNode = node;
    if (node) {
      const dir = node.is_local ? "LOCAL" : "REMOTE";
      const total = formatBytes(node.bytes_sent + node.bytes_recv);
      console.log(
        `[select] ${dir} ${node.ip} (${node.subnet}) | ` +
        `sent=${formatBytes(node.bytes_sent)} recv=${formatBytes(node.bytes_recv)} total=${total} | ` +
        `pkts=${node.packet_count} protos=[${node.protocols.join(",")}]`
      );
    }
  }

  render(): void {
    const now = Date.now();

    if (!this.startupPrinted && this.stats && this.stats.total_packets > 0) {
      this.startupPrinted = true;
      console.log("─────────────────────────────────────────────");
      console.log(" wiregraph");
      console.log("─────────────────────────────────────────────");
      console.log(" drag=rotate  scroll=zoom  click=select");
      console.log("─────────────────────────────────────────────");
      console.log(" EDGE COLORS:");
      const legend = [
        "  HTTP=cyan  TLS=green  DNS=yellow  SSH=orange",
        "  TCP=blue   UDP=purple ICMP=red    NTP=pink",
      ];
      for (const line of legend) console.log(line);
      console.log(" NODE SIZE = traffic volume");
      console.log(" BRIGHT = local    DIM = remote");
      console.log("─────────────────────────────────────────────");
    }

    if (this.stats && now - this.lastLog > 5000) {
      this.lastLog = now;
      const s = this.stats;
      console.log(
        `[stats] ${s.total_packets} pkts | ${formatBytes(s.total_bytes)} | ` +
        `${s.host_count} hosts | ${s.edge_count} edges | ` +
        `${s.packets_per_second.toFixed(1)} pps | ${s.capture_duration.toFixed(0)}s`
      );
    }
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)}KB`;
  if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)}MB`;
  return `${(bytes / 1073741824).toFixed(1)}GB`;
}
