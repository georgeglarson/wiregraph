import * as THREE from "three";
import { Renderer } from "./renderer";
import { Graph } from "./graph";
import { Overlay } from "./overlay";
import { ParticleSystem } from "./particles";
import { Poller } from "./poller";
import { Node, PacketEvent } from "./types";

// State
let paused = false;
let pendingEvents: PacketEvent[] = [];
let latestNodes: Map<string, Node> = new Map();

// Init
const canvas = document.createElement("canvas");
canvas.style.position = "absolute";
canvas.style.top = "0";
canvas.style.left = "0";
document.body.style.margin = "0";
document.body.style.overflow = "hidden";
document.body.style.background = "#0a0a1a";
document.body.appendChild(canvas);

const renderer = new Renderer(canvas);
const graph = new Graph(renderer);
const overlay = new Overlay();
const particles = new ParticleSystem(renderer.scene);
const poller = new Poller();

// Polling callbacks
poller.onTopology = (data) => {
  if (paused) return;
  graph.update(data);
  latestNodes.clear();
  for (const node of data.nodes) {
    latestNodes.set(node.ip, node);
  }
};

poller.onEvents = (events) => {
  if (paused) return;
  pendingEvents.push(...events);
  // Cap pending buffer
  if (pendingEvents.length > 200) {
    pendingEvents = pendingEvents.slice(-200);
  }
};

poller.onStats = (stats) => {
  overlay.setStats(stats);
};

// Node click detection
canvas.addEventListener("click", () => {
  renderer.raycaster.setFromCamera(renderer.mouse, renderer.camera);
  const hit = graph.raycast(renderer.raycaster);
  graph.selectedNode = hit;
  overlay.setSelectedNode(hit);
});

// Keyboard controls
document.addEventListener("keydown", (e) => {
  switch (e.key.toLowerCase()) {
    case " ":
      paused = !paused;
      e.preventDefault();
      break;
    case "r":
      renderer.resetCamera();
      break;
    case "f":
      // Focus on selected node — would need to adjust camera target
      if (graph.selectedNode) {
        const n = graph.selectedNode;
        if (n.x !== undefined && n.y !== undefined && n.z !== undefined) {
          // Camera will look at the node position
          renderer.resetCamera();
        }
      }
      break;
  }
});

// Build node position map for particles
function getNodePositions(): Map<string, THREE.Vector3> {
  const positions = new Map<string, THREE.Vector3>();
  for (const [ip, node] of latestNodes) {
    if (node.x !== undefined && node.y !== undefined && node.z !== undefined) {
      positions.set(ip, new THREE.Vector3(node.x, node.y, node.z));
    }
  }
  return positions;
}

// Render loop
function animate(): void {
  requestAnimationFrame(animate);

  if (!paused) {
    graph.tick();

    // Spawn particles from pending events
    if (pendingEvents.length > 0) {
      const positions = getNodePositions();
      particles.spawn(pendingEvents, positions);
      pendingEvents = [];
    }

    particles.update();
  }

  renderer.render();
  overlay.render();
}

// Start
poller.start();
animate();
