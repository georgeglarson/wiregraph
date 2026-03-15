import { Node, Stats } from "./types";

// Minimal overlay — Canvas2D compositing over WebGPU requires a custom
// shader pipeline in Mystral. For now, log stats to console.
export class Overlay {
  private stats: Stats | null = null;
  private selectedNode: Node | null = null;
  private width: number;
  private height: number;
  private lastLog = 0;

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
      console.log(`[wiregraph] selected: ${node.ip} (${node.subnet}) sent=${node.bytes_sent} recv=${node.bytes_recv} protos=${node.protocols.join(",")}`);
    }
  }

  render(): void {
    // Log stats periodically instead of drawing
    const now = Date.now();
    if (this.stats && now - this.lastLog > 5000) {
      this.lastLog = now;
      const s = this.stats;
      console.log(`[wiregraph] packets=${s.total_packets} hosts=${s.host_count} edges=${s.edge_count} pps=${s.packets_per_second.toFixed(1)}`);
    }
  }
}
