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

interface SimLink {
  source: SimNode;
  target: SimNode;
  edge: Edge;
}

export class Graph {
  private renderer: Renderer;
  private nodeMap = new Map<string, SimNode>();
  private edgeList: SimLink[] = [];
  private simulation: any;

  private nodeMesh: THREE.InstancedMesh | null = null;
  private edgeLines: THREE.LineSegments | null = null;
  private nodeGeometry: THREE.IcosahedronGeometry;
  private nodeMaterial: THREE.MeshPhongMaterial;
  private edgeMaterial: THREE.LineBasicMaterial;

  private dummy = new THREE.Object3D();
  private color = new THREE.Color();

  selectedNode: Node | null = null;

  constructor(renderer: Renderer) {
    this.renderer = renderer;
    this.nodeGeometry = new THREE.IcosahedronGeometry(3, 2);
    this.nodeMaterial = new THREE.MeshPhongMaterial({
      vertexColors: true,
      emissive: new THREE.Color(0x112244),
      emissiveIntensity: 0.3,
    });
    this.edgeMaterial = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.6,
    });

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

    // Update/add nodes
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
      }
    }

    // Remove stale nodes
    for (const ip of this.nodeMap.keys()) {
      if (!incomingIps.has(ip)) {
        this.nodeMap.delete(ip);
        changed = true;
      }
    }

    // Rebuild edges
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

    this.rebuildMeshes();
  }

  private rebuildMeshes(): void {
    const nodes = Array.from(this.nodeMap.values());
    const scene = this.renderer.scene;

    // Remove old meshes
    if (this.nodeMesh) scene.remove(this.nodeMesh);
    if (this.edgeLines) scene.remove(this.edgeLines);

    // Nodes
    if (nodes.length > 0) {
      this.nodeMesh = new THREE.InstancedMesh(this.nodeGeometry, this.nodeMaterial, nodes.length);

      for (let i = 0; i < nodes.length; i++) {
        const n = nodes[i];
        const totalBytes = n.bytes_sent + n.bytes_recv;
        const scale = Math.max(1, Math.log2(totalBytes + 1) * 0.5);

        this.dummy.position.set(n.x ?? 0, n.y ?? 0, n.z ?? 0);
        this.dummy.scale.setScalar(scale);
        this.dummy.updateMatrix();
        this.nodeMesh.setMatrixAt(i, this.dummy.matrix);

        // Color by subnet hash
        const hue = subnetHue(n.subnet);
        const saturation = n.is_local ? 0.7 : 0.4;
        const lightness = 0.6;
        this.color.setHSL(hue, saturation, lightness);
        this.nodeMesh.setColorAt(i, this.color);
      }

      this.nodeMesh.instanceMatrix.needsUpdate = true;
      if (this.nodeMesh.instanceColor) this.nodeMesh.instanceColor.needsUpdate = true;
      scene.add(this.nodeMesh);
    }

    // Edges
    if (this.edgeList.length > 0) {
      const positions = new Float32Array(this.edgeList.length * 6);
      const colors = new Float32Array(this.edgeList.length * 6);

      for (let i = 0; i < this.edgeList.length; i++) {
        const link = this.edgeList[i];
        const si = i * 6;

        positions[si] = link.source.x ?? 0;
        positions[si + 1] = link.source.y ?? 0;
        positions[si + 2] = link.source.z ?? 0;
        positions[si + 3] = link.target.x ?? 0;
        positions[si + 4] = link.target.y ?? 0;
        positions[si + 5] = link.target.z ?? 0;

        const protoColor = PROTOCOL_COLORS[link.edge.protocol] ?? PROTOCOL_COLORS.OTHER;
        this.color.setHex(protoColor);
        const alpha = link.edge.active ? 1.0 : 0.3;
        colors[si] = this.color.r * alpha;
        colors[si + 1] = this.color.g * alpha;
        colors[si + 2] = this.color.b * alpha;
        colors[si + 3] = this.color.r * alpha;
        colors[si + 4] = this.color.g * alpha;
        colors[si + 5] = this.color.b * alpha;
      }

      const geometry = new THREE.BufferGeometry();
      geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
      geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));

      this.edgeLines = new THREE.LineSegments(geometry, this.edgeMaterial);
      scene.add(this.edgeLines);
    }
  }

  tick(): void {
    this.simulation.tick();
    this.rebuildMeshes();
  }

  raycast(raycaster: THREE.Raycaster): Node | null {
    if (!this.nodeMesh) return null;
    const intersects = raycaster.intersectObject(this.nodeMesh);
    if (intersects.length > 0) {
      const idx = intersects[0].instanceId;
      if (idx !== undefined) {
        const nodes = Array.from(this.nodeMap.values());
        return nodes[idx] ?? null;
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
