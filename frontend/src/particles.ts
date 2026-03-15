import * as THREE from "three/webgpu";
import { PacketEvent, PROTOCOL_COLORS } from "./types";

// Particles disabled until InstancedMesh + WebGPU is stable in Mystral.
// Individual meshes work; instancing needs more testing.

export class ParticleSystem {
  constructor(_scene: THREE.Scene) {}

  spawn(
    _events: PacketEvent[],
    _nodePositions: Map<string, THREE.Vector3>,
  ): void {}

  update(): void {}
}
