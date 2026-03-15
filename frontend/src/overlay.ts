import { Node, Stats, PROTOCOL_COLORS } from "./types";

export class Overlay {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private stats: Stats | null = null;
  private selectedNode: Node | null = null;

  constructor() {
    this.canvas = document.createElement("canvas");
    this.canvas.style.position = "absolute";
    this.canvas.style.top = "0";
    this.canvas.style.left = "0";
    this.canvas.style.pointerEvents = "none";
    this.canvas.style.zIndex = "10";
    document.body.appendChild(this.canvas);

    this.ctx = this.canvas.getContext("2d")!;
    this.resize();
    window.addEventListener("resize", () => this.resize());
  }

  private resize(): void {
    this.canvas.width = window.innerWidth;
    this.canvas.height = window.innerHeight;
  }

  setStats(stats: Stats): void {
    this.stats = stats;
  }

  setSelectedNode(node: Node | null): void {
    this.selectedNode = node;
  }

  render(): void {
    const ctx = this.ctx;
    const w = this.canvas.width;
    ctx.clearRect(0, 0, w, this.canvas.height);

    // Stats panel (top-left)
    if (this.stats) {
      ctx.fillStyle = "rgba(10, 10, 30, 0.8)";
      ctx.fillRect(10, 10, 220, 130);
      ctx.strokeStyle = "#334466";
      ctx.strokeRect(10, 10, 220, 130);

      ctx.fillStyle = "#aaccff";
      ctx.font = "bold 13px monospace";
      ctx.fillText("wiregraph", 20, 32);

      ctx.fillStyle = "#88aadd";
      ctx.font = "12px monospace";
      ctx.fillText(`packets: ${this.stats.total_packets.toLocaleString()}`, 20, 52);
      ctx.fillText(`bytes:   ${formatBytes(this.stats.total_bytes)}`, 20, 68);
      ctx.fillText(`hosts:   ${this.stats.host_count}`, 20, 84);
      ctx.fillText(`edges:   ${this.stats.edge_count}`, 20, 100);
      ctx.fillText(`pps:     ${this.stats.packets_per_second.toFixed(1)}`, 20, 116);
      ctx.fillText(`uptime:  ${this.stats.capture_duration.toFixed(1)}s`, 20, 132);
    }

    // Selected node (top-right)
    if (this.selectedNode) {
      const n = this.selectedNode;
      const panelW = 260;
      const x = w - panelW - 10;

      ctx.fillStyle = "rgba(10, 10, 30, 0.8)";
      ctx.fillRect(x, 10, panelW, 120);
      ctx.strokeStyle = "#446688";
      ctx.strokeRect(x, 10, panelW, 120);

      ctx.fillStyle = "#aaccff";
      ctx.font = "bold 13px monospace";
      ctx.fillText(n.ip, x + 10, 32);

      ctx.fillStyle = "#88aadd";
      ctx.font = "12px monospace";
      ctx.fillText(`subnet:  ${n.subnet}`, x + 10, 52);
      ctx.fillText(`sent:    ${formatBytes(n.bytes_sent)}`, x + 10, 68);
      ctx.fillText(`recv:    ${formatBytes(n.bytes_recv)}`, x + 10, 84);
      ctx.fillText(`packets: ${n.packet_count.toLocaleString()}`, x + 10, 100);
      ctx.fillText(`protos:  ${n.protocols.join(", ")}`, x + 10, 116);
    }

    // Protocol legend (bottom-left)
    const legendEntries = Object.entries(PROTOCOL_COLORS);
    const legendH = legendEntries.length * 18 + 24;
    const legendY = this.canvas.height - legendH - 10;

    ctx.fillStyle = "rgba(10, 10, 30, 0.8)";
    ctx.fillRect(10, legendY, 120, legendH);
    ctx.strokeStyle = "#334466";
    ctx.strokeRect(10, legendY, 120, legendH);

    ctx.fillStyle = "#aaccff";
    ctx.font = "bold 11px monospace";
    ctx.fillText("protocols", 20, legendY + 16);

    ctx.font = "11px monospace";
    legendEntries.forEach(([name, hex], i) => {
      const y = legendY + 34 + i * 18;
      ctx.fillStyle = `#${hex.toString(16).padStart(6, "0")}`;
      ctx.fillRect(20, y - 8, 10, 10);
      ctx.fillStyle = "#88aadd";
      ctx.fillText(name, 36, y);
    });

    // Controls hint (bottom-right)
    ctx.fillStyle = "rgba(10, 10, 30, 0.6)";
    ctx.fillRect(w - 210, this.canvas.height - 50, 200, 40);
    ctx.fillStyle = "#556688";
    ctx.font = "11px monospace";
    ctx.fillText("drag:rotate  wheel:zoom", w - 200, this.canvas.height - 30);
    ctx.fillText("click:select  R:reset  F:focus", w - 200, this.canvas.height - 16);
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
  return `${(bytes / 1073741824).toFixed(1)} GB`;
}
