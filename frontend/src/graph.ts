import * as THREE from "three/webgpu";
// @ts-ignore — d3-force-3d has no types
import { forceSimulation, forceLink, forceManyBody, forceCenter } from "d3-force-3d";
import { Node, Edge, TopologyResponse, PROTOCOL_COLORS } from "./types";
import { Renderer } from "./renderer";

interface SimNode extends Node {
  x: number;
  y: number;
  z: number;
}

const MAX_EDGES = 256;

export class Graph {
  private renderer: Renderer;
  private nodeMap = new Map<string, SimNode>();
  private edgeList: { source: SimNode; target: SimNode; edge: Edge }[] = [];
  private simulation: any;

  private nodeGroup = new THREE.Group();
  private nodeSpheres = new Map<string, THREE.Mesh>();

  // Pre-allocated edge geometry — update positions in place, no realloc
  private edgePositions: Float32Array;
  private edgeGeo: THREE.BufferGeometry;
  private edgeLines: THREE.LineSegments;
  private activeEdgeCount = 0;

  selectedNode: Node | null = null;

  constructor(renderer: Renderer) {
    this.renderer = renderer;
    renderer.scene.add(this.nodeGroup);

    // Pre-allocate edge buffer for MAX_EDGES lines
    this.edgePositions = new Float32Array(MAX_EDGES * 6);
    this.edgeGeo = new THREE.BufferGeometry();
    const posAttr = new THREE.BufferAttribute(this.edgePositions, 3);
    posAttr.setUsage(THREE.DynamicDrawUsage);
    this.edgeGeo.setAttribute("position", posAttr);
    this.edgeGeo.setDrawRange(0, 0);

    this.edgeLines = new THREE.LineSegments(
      this.edgeGeo,
      new THREE.LineBasicMaterial({ color: 0x4488ff, transparent: true, opacity: 0.5 }),
    );
    renderer.scene.add(this.edgeLines);

    this.simulation = forceSimulation()
      .numDimensions(3)
      .force("charge", forceManyBody().strength(-80))
      .force("center", forceCenter())
      .force(
        "link",
        forceLink()
          .id((d: any) => d.ip)
          .distance(60),
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

        // Create sphere mesh for this node
        const totalBytes = node.bytes_sent + node.bytes_recv;
        const radius = Math.max(1.5, Math.log2(totalBytes + 1) * 0.6);
        const hue = subnetHue(node.subnet);
        const color = new THREE.Color().setHSL(hue, node.is_local ? 0.7 : 0.4, 0.6);

        const geo = new THREE.IcosahedronGeometry(radius, 1);
        const mat = new THREE.MeshStandardMaterial({
          color,
          emissive: color,
          emissiveIntensity: 0.2,
          metalness: 0.3,
          roughness: 0.6,
        });
        const mesh = new THREE.Mesh(geo, mat);
        mesh.position.set(simNode.x, simNode.y, simNode.z);
        this.nodeGroup.add(mesh);
        this.nodeSpheres.set(node.ip, mesh);

        console.log(`[wiregraph] + node ${node.ip} r=${radius.toFixed(1)}`);
      }
    }

    // Remove stale nodes
    for (const ip of this.nodeMap.keys()) {
      if (!incomingIps.has(ip)) {
        this.nodeMap.delete(ip);
        const mesh = this.nodeSpheres.get(ip);
        if (mesh) {
          this.nodeGroup.remove(mesh);
          this.nodeSpheres.delete(ip);
        }
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
    this.simulation.tick();

    // Update node mesh positions
    for (const [ip, simNode] of this.nodeMap) {
      const mesh = this.nodeSpheres.get(ip);
      if (mesh) {
        mesh.position.set(simNode.x, simNode.y, simNode.z);
      }
    }

    // Update edge positions in pre-allocated buffer
    const count = Math.min(this.edgeList.length, MAX_EDGES);
    for (let i = 0; i < count; i++) {
      const link = this.edgeList[i];
      const si = i * 6;
      this.edgePositions[si] = link.source.x;
      this.edgePositions[si + 1] = link.source.y;
      this.edgePositions[si + 2] = link.source.z;
      this.edgePositions[si + 3] = link.target.x;
      this.edgePositions[si + 4] = link.target.y;
      this.edgePositions[si + 5] = link.target.z;
    }

    const posAttr = this.edgeGeo.getAttribute("position") as THREE.BufferAttribute;
    posAttr.needsUpdate = true;
    this.edgeGeo.setDrawRange(0, count * 2);
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
