import * as THREE from "three";
import { PacketEvent, PROTOCOL_COLORS } from "./types";

const MAX_ACTIVE = 100;
const PARTICLE_LIFETIME_MS = 300;

interface Particle {
  start: THREE.Vector3;
  end: THREE.Vector3;
  color: THREE.Color;
  born: number;
  size: number;
}

export class ParticleSystem {
  private particles: Particle[] = [];
  private mesh: THREE.InstancedMesh;
  private dummy = new THREE.Object3D();
  private color = new THREE.Color();
  private pool: number;

  constructor(scene: THREE.Scene, pool = 200) {
    this.pool = pool;

    const geo = new THREE.SphereGeometry(0.8, 4, 4);
    const mat = new THREE.MeshBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.9,
    });

    this.mesh = new THREE.InstancedMesh(geo, mat, pool);
    this.mesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);

    // Start all instances off-screen
    this.dummy.position.set(99999, 99999, 99999);
    this.dummy.scale.setScalar(0);
    this.dummy.updateMatrix();
    for (let i = 0; i < pool; i++) {
      this.mesh.setMatrixAt(i, this.dummy.matrix);
    }

    scene.add(this.mesh);
  }

  spawn(
    events: PacketEvent[],
    nodePositions: Map<string, THREE.Vector3>,
  ): void {
    for (const event of events) {
      if (this.particles.length >= MAX_ACTIVE) break;

      const startPos = nodePositions.get(event.source);
      const endPos = nodePositions.get(event.target);
      if (!startPos || !endPos) continue;

      const protoColor = PROTOCOL_COLORS[event.protocol] ?? PROTOCOL_COLORS.OTHER;
      const size = Math.max(0.5, Math.min(2, Math.log2(event.bytes + 1) * 0.3));

      this.particles.push({
        start: startPos.clone(),
        end: endPos.clone(),
        color: new THREE.Color(protoColor),
        born: performance.now(),
        size,
      });
    }
  }

  update(): void {
    const now = performance.now();

    // Remove expired
    this.particles = this.particles.filter((p) => now - p.born < PARTICLE_LIFETIME_MS);

    // Update instances
    for (let i = 0; i < this.pool; i++) {
      if (i < this.particles.length) {
        const p = this.particles[i];
        const t = (now - p.born) / PARTICLE_LIFETIME_MS;

        this.dummy.position.lerpVectors(p.start, p.end, t);
        this.dummy.scale.setScalar(p.size * (1 - t * 0.5));
        this.dummy.updateMatrix();
        this.mesh.setMatrixAt(i, this.dummy.matrix);
        this.mesh.setColorAt(i, p.color);
      } else {
        this.dummy.position.set(99999, 99999, 99999);
        this.dummy.scale.setScalar(0);
        this.dummy.updateMatrix();
        this.mesh.setMatrixAt(i, this.dummy.matrix);
      }
    }

    this.mesh.instanceMatrix.needsUpdate = true;
    if (this.mesh.instanceColor) this.mesh.instanceColor.needsUpdate = true;
  }
}
