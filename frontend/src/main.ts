import * as THREE from "three/webgpu";
import { Renderer } from "./renderer";
import { Graph } from "./graph";
import { Overlay } from "./overlay";
import { ParticleSystem } from "./particles";
import { Poller } from "./poller";
import { Node, PacketEvent } from "./types";

declare const canvas: HTMLCanvasElement;

async function main(): Promise<void> {
  console.log("[wiregraph] starting");

  let paused = false;
  let pendingEvents: PacketEvent[] = [];
  let latestNodes: Map<string, Node> = new Map();

  const renderer = await Renderer.create();
  console.log("[wiregraph] renderer ready");

  const graph = new Graph(renderer);
  const overlay = new Overlay(renderer.width, renderer.height);
  const particles = new ParticleSystem(renderer.scene);
  const poller = new Poller();

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
    if (pendingEvents.length > 200) {
      pendingEvents = pendingEvents.slice(-200);
    }
  };

  poller.onStats = (stats) => {
    overlay.setStats(stats);
  };

  canvas.addEventListener("click", (e: MouseEvent) => {
    renderer.raycaster.setFromCamera(renderer.mouse, renderer.camera);
    const hit = graph.raycast(renderer.raycaster);
    graph.selectedNode = hit;
    overlay.setSelectedNode(hit);
  });

  canvas.addEventListener("keydown", (e: KeyboardEvent) => {
    switch (e.key.toLowerCase()) {
      case " ":
        paused = !paused;
        break;
      case "r":
        renderer.resetCamera();
        break;
      case "f":
        if (graph.selectedNode) {
          renderer.resetCamera();
        }
        break;
    }
  });

  function getNodePositions(): Map<string, THREE.Vector3> {
    const positions = new Map<string, THREE.Vector3>();
    for (const [ip, node] of latestNodes) {
      if (node.x !== undefined && node.y !== undefined && node.z !== undefined) {
        positions.set(ip, new THREE.Vector3(node.x, node.y, node.z));
      }
    }
    return positions;
  }

  let frameCount = 0;
  function animate(): void {
    frameCount++;

    if (!paused) {
      graph.tick();

      if (pendingEvents.length > 0) {
        const positions = getNodePositions();
        particles.spawn(pendingEvents, positions);
        pendingEvents = [];
      }

      particles.update();
    }

    renderer.render();
    overlay.render();

    if (frameCount % 120 === 0) {
      console.log(`[wiregraph] frame ${frameCount}, nodes=${latestNodes.size}`);
    }

    requestAnimationFrame(animate);
  }

  poller.start();
  animate();
  console.log("[wiregraph] running");
}

main().catch((e) => {
  console.error("[wiregraph] init failed:", e.message);
  console.error("[wiregraph] stack:", e.stack);
});
