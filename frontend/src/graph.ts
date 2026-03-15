import * as THREE from "three/webgpu";
// @ts-ignore — d3-force-3d has no types
import { forceSimulation, forceLink, forceManyBody, forceCenter } from "d3-force-3d";
import { Node, Edge, TopologyResponse, PROTOCOL_COLORS } from "./types";
import { Renderer } from "./renderer";
// Labels disabled — creating CanvasTexture mid-render crashes Mystral's Dawn
// import { createLabel } from "./labels";

interface SimNode extends Node {
  x: number;
  y: number;
  z: number;
}

const MAX_EDGES_PER_PROTO = 64;
const EDGE_PROTOS = ["HTTP", "TLS", "DNS", "SSH", "TCP", "UDP", "ICMP", "NTP", "SMTP", "DHCP", "OTHER"] as const;

export class Graph {
  private renderer: Renderer;
  private nodeMap = new Map<string, SimNode>();
  private edgeList: { source: SimNode; target: SimNode; edge: Edge }[] = [];
  private simulation: any;

  private nodeGroup = new THREE.Group();
  private labelGroup = new THREE.Group();
  private nodeSpheres = new Map<string, THREE.Mesh>();
  private nodeLabels = new Map<string, THREE.Sprite>();

  // Queue of nodes waiting to have their meshes created (done in tick, not async callback)
  private pendingNodes: SimNode[] = [];

  private protoEdges = new Map<string, {
    positions: Float32Array;
    geo: THREE.BufferGeometry;
    lines: THREE.LineSegments;
    count: number;
  }>();

  selectedNode: Node | null = null;

  constructor(renderer: Renderer) {
    this.renderer = renderer;
    renderer.scene.add(this.nodeGroup);
    renderer.scene.add(this.labelGroup);

    for (const proto of EDGE_PROTOS) {
      const positions = new Float32Array(MAX_EDGES_PER_PROTO * 6);
      const geo = new THREE.BufferGeometry();
      const posAttr = new THREE.BufferAttribute(positions, 3);
      posAttr.setUsage(THREE.DynamicDrawUsage);
      geo.setAttribute("position", posAttr);
      geo.setDrawRange(0, 0);

      const hex = PROTOCOL_COLORS[proto] ?? PROTOCOL_COLORS.OTHER;
      const lines = new THREE.LineSegments(
        geo,
        new THREE.LineBasicMaterial({ color: hex, transparent: true, opacity: 0.6 }),
      );
      renderer.scene.add(lines);
      this.protoEdges.set(proto, { positions, geo, lines, count: 0 });
    }

    this.simulation = forceSimulation()
      .numDimensions(3)
      .force("charge", forceManyBody().strength(-100))
      .force("center", forceCenter())
      .force(
        "link",
        forceLink()
          .id((d: any) => d.ip)
          .distance(70),
      )
      .alphaDecay(0.02)
      .velocityDecay(0.3);
  }

  update(data: TopologyResponse): void {
    let changed = false;

    const incomingIps = new Set<string>();
    for (const node of data.nodes) {
      incomingIps.add(node.ip);
      const existing = this.nodeMap.get(node.ip);
      if (existing) {
        existing.bytes_sent = node.bytes_sent;
        existing.bytes_recv = node.bytes_recv;
        existing.packet_count = node.packet_count;
        existing.protocols = node.protocols;
        existing.last_seen = node.last_seen;
      } else {
        const simNode: SimNode = {
          ...node,
          x: (Math.random() - 0.5) * 100,
          y: (Math.random() - 0.5) * 100,
          z: (Math.random() - 0.5) * 100,
        };
        this.nodeMap.set(node.ip, simNode);
        changed = true;
        // Queue for mesh creation in tick() — not here in async callback
        this.pendingNodes.push(simNode);
      }
    }

    // Remove stale
    for (const ip of this.nodeMap.keys()) {
      if (!incomingIps.has(ip)) {
        this.nodeMap.delete(ip);
        const mesh = this.nodeSpheres.get(ip);
        if (mesh) { this.nodeGroup.remove(mesh); this.nodeSpheres.delete(ip); }
        const label = this.nodeLabels.get(ip);
        if (label) { this.labelGroup.remove(label); this.nodeLabels.delete(ip); }
        changed = true;
      }
    }

    // Rebuild edge list
    this.edgeList = [];
    for (const edge of data.edges) {
      const src = this.nodeMap.get(edge.source);
      const tgt = this.nodeMap.get(edge.target);
      if (src && tgt) {
        this.edgeList.push({ source: src, target: tgt, edge });
      }
    }

    if (changed) {
      const nodes = Array.from(this.nodeMap.values());
      this.simulation.nodes(nodes);
      this.simulation.force("link").links(
        this.edgeList.map((l) => ({
          source: l.source.ip,
          target: l.target.ip,
        })),
      );
      this.simulation.alpha(0.5).restart();
    }
  }

  tick(): void {
    // Create meshes for queued nodes (safe — we're in the animation frame)
    if (this.pendingNodes.length > 0) {
      for (const simNode of this.pendingNodes) {
        this.createNodeMesh(simNode);
      }
      this.pendingNodes = [];
    }

    this.simulation.tick();

    // Update positions
    for (const [ip, simNode] of this.nodeMap) {
      const mesh = this.nodeSpheres.get(ip);
      if (mesh) mesh.position.set(simNode.x, simNode.y, simNode.z);
      const label = this.nodeLabels.get(ip);
      if (label) label.position.set(simNode.x, simNode.y + 8, simNode.z);
    }

    // Update edges
    for (const pe of this.protoEdges.values()) {
      pe.count = 0;
    }

    for (const link of this.edgeList) {
      const proto = link.edge.protocol;
      let pe = this.protoEdges.get(proto);
      if (!pe) pe = this.protoEdges.get("OTHER")!;
      if (pe.count >= MAX_EDGES_PER_PROTO) continue;

      const si = pe.count * 6;
      pe.positions[si] = link.source.x;
      pe.positions[si + 1] = link.source.y;
      pe.positions[si + 2] = link.source.z;
      pe.positions[si + 3] = link.target.x;
      pe.positions[si + 4] = link.target.y;
      pe.positions[si + 5] = link.target.z;
      pe.count++;
    }

    for (const pe of this.protoEdges.values()) {
      const posAttr = pe.geo.getAttribute("position") as THREE.BufferAttribute;
      posAttr.needsUpdate = true;
      pe.geo.setDrawRange(0, pe.count * 2);
    }
  }

  private createNodeMesh(node: SimNode): void {
    const totalBytes = node.bytes_sent + node.bytes_recv;
    const baseRadius = node.is_local ? 3.0 : 2.0;
    const radius = Math.max(baseRadius, Math.log2(totalBytes + 1) * 0.7);

    const hue = subnetHue(node.subnet);
    const sat = node.is_local ? 0.8 : 0.3;
    const lum = node.is_local ? 0.65 : 0.5;
    const color = new THREE.Color().setHSL(hue, sat, lum);

    const geo = new THREE.IcosahedronGeometry(radius, 1);
    const mat = new THREE.MeshStandardMaterial({
      color,
      emissive: color,
      emissiveIntensity: node.is_local ? 0.3 : 0.1,
      metalness: 0.3,
      roughness: 0.5,
    });
    const mesh = new THREE.Mesh(geo, mat);
    mesh.position.set(node.x, node.y, node.z);
    this.nodeGroup.add(mesh);
    this.nodeSpheres.set(node.ip, mesh);

    console.log(`[wiregraph] + ${node.is_local ? "local" : "remote"} ${node.ip}`);
  }

  raycast(raycaster: THREE.Raycaster): Node | null {
    const intersects = raycaster.intersectObjects(this.nodeGroup.children);
    if (intersects.length > 0) {
      const hitMesh = intersects[0].object;
      for (const [ip, mesh] of this.nodeSpheres) {
        if (mesh === hitMesh) {
          return this.nodeMap.get(ip) ?? null;
        }
      }
    }
    return null;
  }
}

function subnetHue(subnet: string): number {
  let hash = 0;
  for (let i = 0; i < subnet.length; i++) {
    hash = (hash * 31 + subnet.charCodeAt(i)) | 0;
  }
  return Math.abs(hash % 360) / 360;
}
