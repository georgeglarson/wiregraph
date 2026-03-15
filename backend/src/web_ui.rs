pub const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>wiregraph</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: #0a0a1a; color: #88aadd; font: 13px/1.4 monospace; overflow: hidden; }
canvas { position: absolute; top: 0; left: 0; }

#hud {
  position: absolute; top: 0; left: 0; right: 0; bottom: 0;
  pointer-events: none; z-index: 10;
}

#stats {
  position: absolute; top: 12px; left: 12px;
  background: rgba(10,10,30,0.85); border: 1px solid #334466;
  padding: 10px 14px; border-radius: 4px;
}
#stats h2 { color: #aaccff; font-size: 14px; margin-bottom: 6px; }
#stats div { margin: 2px 0; }

#selection {
  position: absolute; top: 12px; right: 12px;
  background: rgba(10,10,30,0.85); border: 1px solid #446688;
  padding: 10px 14px; border-radius: 4px; display: none; min-width: 220px;
}
#selection h3 { color: #aaccff; font-size: 13px; margin-bottom: 4px; }
#selection div { margin: 2px 0; }
#selection .tag { display: inline-block; padding: 1px 6px; border-radius: 3px; font-size: 11px; margin: 1px 2px; }

#legend {
  position: absolute; bottom: 12px; left: 12px;
  background: rgba(10,10,30,0.85); border: 1px solid #334466;
  padding: 8px 12px; border-radius: 4px;
}
#legend h4 { color: #aaccff; font-size: 11px; margin-bottom: 4px; }
.legend-row { display: flex; align-items: center; margin: 2px 0; font-size: 11px; }
.legend-dot { width: 10px; height: 10px; border-radius: 50%; margin-right: 6px; display: inline-block; }

#controls {
  position: absolute; bottom: 12px; right: 12px;
  background: rgba(10,10,30,0.6); padding: 6px 10px; border-radius: 4px;
  font-size: 11px; color: #556688;
}
</style>
</head>
<body>
<canvas id="c"></canvas>
<div id="hud">
  <div id="stats">
    <h2>wiregraph</h2>
    <div>packets: <span id="s-pkts">0</span></div>
    <div>bytes: <span id="s-bytes">0</span></div>
    <div>hosts: <span id="s-hosts">0</span></div>
    <div>edges: <span id="s-edges">0</span></div>
    <div>pps: <span id="s-pps">0</span></div>
    <div>uptime: <span id="s-uptime">0</span>s</div>
  </div>
  <div id="selection">
    <h3 id="sel-ip"></h3>
    <div>subnet: <span id="sel-subnet"></span></div>
    <div>sent: <span id="sel-sent"></span></div>
    <div>recv: <span id="sel-recv"></span></div>
    <div>packets: <span id="sel-pkts"></span></div>
    <div id="sel-protos"></div>
  </div>
  <div id="legend">
    <h4>protocols</h4>
    <div class="legend-row"><span class="legend-dot" style="background:#00ffff"></span>HTTP</div>
    <div class="legend-row"><span class="legend-dot" style="background:#00ff88"></span>TLS</div>
    <div class="legend-row"><span class="legend-dot" style="background:#ffff00"></span>DNS</div>
    <div class="legend-row"><span class="legend-dot" style="background:#ff8800"></span>SSH</div>
    <div class="legend-row"><span class="legend-dot" style="background:#4488ff"></span>TCP</div>
    <div class="legend-row"><span class="legend-dot" style="background:#aa44ff"></span>UDP</div>
    <div class="legend-row"><span class="legend-dot" style="background:#ff4444"></span>ICMP</div>
    <div class="legend-row"><span class="legend-dot" style="background:#ff88ff"></span>NTP</div>
  </div>
  <div id="controls">
    drag: rotate &nbsp; scroll: zoom &nbsp; click: select<br>
    space: pause &nbsp; R: reset
  </div>
</div>

<script type="importmap">
{
  "imports": {
    "three": "https://cdn.jsdelivr.net/npm/three@0.182.0/build/three.module.js",
    "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.182.0/examples/jsm/"
  }
}
</script>
<script type="module">
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

const PROTO_COLORS = {
  HTTP: 0x00ffff, TLS: 0x00ff88, DNS: 0xffff00, SSH: 0xff8800,
  TCP: 0x4488ff, UDP: 0xaa44ff, ICMP: 0xff4444, DHCP: 0x88ff44,
  NTP: 0xff88ff, SMTP: 0xff6644, OTHER: 0x888888,
};

// --- Three.js setup ---
const canvas = document.getElementById('c');
const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.setPixelRatio(window.devicePixelRatio);

const scene = new THREE.Scene();
scene.background = new THREE.Color(0x0a0a1a);

const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 1, 5000);
camera.position.set(150, 120, 200);

const controls = new OrbitControls(camera, canvas);
controls.enableDamping = true;
controls.dampingFactor = 0.08;
controls.target.set(0, 0, 0);

scene.add(new THREE.AmbientLight(0x334466, 1.5));
const p1 = new THREE.PointLight(0xffffff, 2, 1000);
p1.position.set(100, 200, 150);
scene.add(p1);
const p2 = new THREE.PointLight(0x4488ff, 1, 800);
p2.position.set(-150, -100, -100);
scene.add(p2);

const grid = new THREE.GridHelper(400, 20, 0x222244, 0x111133);
grid.position.y = -80;
scene.add(grid);

window.addEventListener('resize', () => {
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(window.innerWidth, window.innerHeight);
});

// --- Force layout (simple spring simulation, no d3 dependency) ---
const nodes = new Map(); // ip -> { x, y, z, vx, vy, vz, mesh, label, data }
const edges = [];        // [{ srcIp, dstIp, edge }]

function subnetHue(subnet) {
  let h = 0;
  for (let i = 0; i < subnet.length; i++) h = (h * 31 + subnet.charCodeAt(i)) | 0;
  return (Math.abs(h % 360)) / 360;
}

function createNodeMesh(node) {
  const totalBytes = node.bytes_sent + node.bytes_recv;
  const r = Math.max(node.is_local ? 3 : 2, Math.log2(totalBytes + 1) * 0.7);
  const hue = subnetHue(node.subnet);
  const color = new THREE.Color().setHSL(hue, node.is_local ? 0.8 : 0.3, node.is_local ? 0.65 : 0.5);

  const mesh = new THREE.Mesh(
    new THREE.IcosahedronGeometry(r, 1),
    new THREE.MeshStandardMaterial({
      color, emissive: color,
      emissiveIntensity: node.is_local ? 0.3 : 0.1,
      metalness: 0.3, roughness: 0.5,
    })
  );
  scene.add(mesh);

  // Text label sprite
  const c = document.createElement('canvas');
  c.width = 256; c.height = 64;
  const ctx = c.getContext('2d');
  ctx.fillStyle = node.is_local ? '#88ddff' : '#667799';
  ctx.font = 'bold 26px monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(node.ip, 128, 32);
  const tex = new THREE.CanvasTexture(c);
  const label = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex, transparent: true, depthTest: false }));
  label.scale.set(28, 7, 1);
  scene.add(label);

  return { mesh, label, radius: r };
}

// Edge lines: one LineSegments per protocol
const edgeGroups = {};
for (const [proto, hex] of Object.entries(PROTO_COLORS)) {
  const geo = new THREE.BufferGeometry();
  const pos = new Float32Array(256 * 6);
  geo.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
  geo.setDrawRange(0, 0);
  const lines = new THREE.LineSegments(geo, new THREE.LineBasicMaterial({
    color: hex, transparent: true, opacity: 0.6,
  }));
  scene.add(lines);
  edgeGroups[proto] = { geo, pos, lines, count: 0 };
}

// --- Polling ---
let lastEventTs = 0;

async function pollTopology() {
  try {
    const res = await fetch('/api/topology');
    if (!res.ok) return;
    const data = await res.json();
    updateGraph(data);
  } catch {}
  setTimeout(pollTopology, 1000);
}

async function pollStats() {
  try {
    const res = await fetch('/api/stats');
    if (!res.ok) return;
    const s = await res.json();
    document.getElementById('s-pkts').textContent = s.total_packets.toLocaleString();
    document.getElementById('s-bytes').textContent = fmtBytes(s.total_bytes);
    document.getElementById('s-hosts').textContent = s.host_count;
    document.getElementById('s-edges').textContent = s.edge_count;
    document.getElementById('s-pps').textContent = s.packets_per_second.toFixed(1);
    document.getElementById('s-uptime').textContent = s.capture_duration.toFixed(0);
  } catch {}
  setTimeout(pollStats, 1000);
}

function fmtBytes(b) {
  if (b < 1024) return b + ' B';
  if (b < 1048576) return (b/1024).toFixed(1) + ' KB';
  if (b < 1073741824) return (b/1048576).toFixed(1) + ' MB';
  return (b/1073741824).toFixed(1) + ' GB';
}

function updateGraph(data) {
  const incoming = new Set();

  for (const n of data.nodes) {
    incoming.add(n.ip);
    if (nodes.has(n.ip)) {
      nodes.get(n.ip).data = n;
    } else {
      const { mesh, label, radius } = createNodeMesh(n);
      const x = (Math.random() - 0.5) * 100;
      const y = (Math.random() - 0.5) * 100;
      const z = (Math.random() - 0.5) * 100;
      nodes.set(n.ip, { x, y, z, vx: 0, vy: 0, vz: 0, mesh, label, radius, data: n });
    }
  }

  // Remove stale
  for (const [ip, n] of nodes) {
    if (!incoming.has(ip)) {
      scene.remove(n.mesh);
      scene.remove(n.label);
      nodes.delete(ip);
    }
  }

  // Update edges
  edges.length = 0;
  for (const e of data.edges) {
    if (nodes.has(e.source) && nodes.has(e.target)) {
      edges.push({ srcIp: e.source, dstIp: e.target, edge: e });
    }
  }
}

// --- Simple force simulation ---
function simTick() {
  const alpha = 0.3;
  const repulsion = -600;
  const linkDist = 70;
  const linkStrength = 0.02;
  const centering = 0.01;
  const damping = 0.85;

  const nodeArr = [...nodes.values()];

  // Repulsion (charge)
  for (let i = 0; i < nodeArr.length; i++) {
    for (let j = i + 1; j < nodeArr.length; j++) {
      const a = nodeArr[i], b = nodeArr[j];
      let dx = b.x - a.x, dy = b.y - a.y, dz = b.z - a.z;
      let dist = Math.sqrt(dx*dx + dy*dy + dz*dz) || 1;
      const force = repulsion / (dist * dist);
      const fx = dx / dist * force * alpha;
      const fy = dy / dist * force * alpha;
      const fz = dz / dist * force * alpha;
      a.vx -= fx; a.vy -= fy; a.vz -= fz;
      b.vx += fx; b.vy += fy; b.vz += fz;
    }
  }

  // Link attraction
  for (const { srcIp, dstIp } of edges) {
    const a = nodes.get(srcIp), b = nodes.get(dstIp);
    if (!a || !b) continue;
    let dx = b.x - a.x, dy = b.y - a.y, dz = b.z - a.z;
    let dist = Math.sqrt(dx*dx + dy*dy + dz*dz) || 1;
    const force = (dist - linkDist) * linkStrength * alpha;
    const fx = dx / dist * force;
    const fy = dy / dist * force;
    const fz = dz / dist * force;
    a.vx += fx; a.vy += fy; a.vz += fz;
    b.vx -= fx; b.vy -= fy; b.vz -= fz;
  }

  // Centering + velocity update
  for (const n of nodeArr) {
    n.vx -= n.x * centering;
    n.vy -= n.y * centering;
    n.vz -= n.z * centering;
    n.vx *= damping; n.vy *= damping; n.vz *= damping;
    n.x += n.vx; n.y += n.vy; n.z += n.vz;

    n.mesh.position.set(n.x, n.y, n.z);
    n.label.position.set(n.x, n.y + n.radius + 5, n.z);
  }

  // Update edge line buffers
  for (const eg of Object.values(edgeGroups)) { eg.count = 0; }

  for (const { srcIp, dstIp, edge } of edges) {
    const a = nodes.get(srcIp), b = nodes.get(dstIp);
    if (!a || !b) continue;
    const proto = edge.protocol;
    const eg = edgeGroups[proto] || edgeGroups.OTHER;
    if (eg.count >= 256) continue;
    const i = eg.count * 6;
    eg.pos[i] = a.x; eg.pos[i+1] = a.y; eg.pos[i+2] = a.z;
    eg.pos[i+3] = b.x; eg.pos[i+4] = b.y; eg.pos[i+5] = b.z;
    eg.count++;
  }

  for (const eg of Object.values(edgeGroups)) {
    eg.geo.attributes.position.needsUpdate = true;
    eg.geo.setDrawRange(0, eg.count * 2);
  }
}

// --- Click / selection ---
const raycaster = new THREE.Raycaster();
const mouse = new THREE.Vector2();

canvas.addEventListener('click', (e) => {
  mouse.x = (e.clientX / window.innerWidth) * 2 - 1;
  mouse.y = -(e.clientY / window.innerHeight) * 2 + 1;
  raycaster.setFromCamera(mouse, camera);

  const meshes = [...nodes.values()].map(n => n.mesh);
  const hits = raycaster.intersectObjects(meshes);

  if (hits.length > 0) {
    for (const [ip, n] of nodes) {
      if (n.mesh === hits[0].object) {
        showSelection(n.data);
        return;
      }
    }
  }
  hideSelection();
});

function showSelection(n) {
  const el = document.getElementById('selection');
  el.style.display = 'block';
  document.getElementById('sel-ip').textContent = (n.is_local ? '(local) ' : '') + n.ip;
  document.getElementById('sel-subnet').textContent = n.subnet;
  document.getElementById('sel-sent').textContent = fmtBytes(n.bytes_sent);
  document.getElementById('sel-recv').textContent = fmtBytes(n.bytes_recv);
  document.getElementById('sel-pkts').textContent = n.packet_count.toLocaleString();
  const protos = document.getElementById('sel-protos');
  protos.innerHTML = '';
  for (const p of n.protocols) {
    const hex = (PROTO_COLORS[p] || PROTO_COLORS.OTHER).toString(16).padStart(6, '0');
    protos.innerHTML += `<span class="tag" style="background:#${hex}33;color:#${hex}">${p}</span>`;
  }
}

function hideSelection() {
  document.getElementById('selection').style.display = 'none';
}

// --- Keyboard ---
let paused = false;
document.addEventListener('keydown', (e) => {
  if (e.key === ' ') { paused = !paused; e.preventDefault(); }
  if (e.key === 'r' || e.key === 'R') { camera.position.set(150, 120, 200); controls.target.set(0,0,0); }
});

// --- Render loop ---
function animate() {
  requestAnimationFrame(animate);
  if (!paused) simTick();
  controls.update();
  renderer.render(scene, camera);
}

pollTopology();
pollStats();
animate();
</script>
</body>
</html>
"##;
