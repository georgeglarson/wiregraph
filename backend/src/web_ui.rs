pub const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>wiregraph</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  background: #0b0e17; color: #c0ccdd; font: 13px/1.5 'JetBrains Mono', 'Fira Code', monospace;
  display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: auto 1fr 1fr;
  gap: 8px; padding: 8px; height: 100vh;
}
.panel {
  background: #111827; border: 1px solid #1e293b; border-radius: 6px;
  padding: 12px; overflow: auto;
}
.panel h2 {
  color: #7dd3fc; font-size: 12px; text-transform: uppercase;
  letter-spacing: 1px; margin-bottom: 8px; border-bottom: 1px solid #1e293b;
  padding-bottom: 4px;
}

/* Header bar */
#header {
  grid-column: 1 / -1; display: flex; align-items: center; justify-content: space-between;
  background: #111827; border: 1px solid #1e293b; border-radius: 6px; padding: 8px 16px;
}
#header h1 { color: #7dd3fc; font-size: 16px; font-weight: bold; }
#header .stats { display: flex; gap: 20px; font-size: 12px; color: #64748b; }
#header .stats span { color: #94a3b8; }

/* Hosts table */
#hosts { grid-column: 1; }
#hosts table { width: 100%; border-collapse: collapse; font-size: 12px; }
#hosts th {
  text-align: left; color: #64748b; font-weight: normal; padding: 4px 8px;
  border-bottom: 1px solid #1e293b; cursor: pointer; user-select: none;
}
#hosts th:hover { color: #7dd3fc; }
#hosts td { padding: 4px 8px; border-bottom: 1px solid #0f172a; }
#hosts tr:hover td { background: #1e293b; }
#hosts tr.selected td { background: #1e3a5f; }
.ip { color: #e2e8f0; font-weight: bold; }
.local { color: #34d399; }
.remote { color: #94a3b8; }
.bar-cell { position: relative; }
.bar-fill {
  position: absolute; left: 0; top: 2px; bottom: 2px;
  border-radius: 2px; opacity: 0.3;
}
.bar-text { position: relative; z-index: 1; }
.proto-tag {
  display: inline-block; padding: 0 4px; border-radius: 3px;
  font-size: 10px; margin: 0 1px; line-height: 16px;
}

/* Matrix */
#matrix { grid-column: 2; }
#matrix-canvas { width: 100%; height: calc(100% - 30px); display: block; }

/* Protocols */
#protocols { grid-column: 1; }
#proto-bars { margin-top: 4px; }
.proto-row { display: flex; align-items: center; margin: 3px 0; }
.proto-label { width: 50px; font-size: 11px; text-align: right; padding-right: 8px; }
.proto-bar-bg { flex: 1; height: 16px; background: #1e293b; border-radius: 3px; overflow: hidden; position: relative; }
.proto-bar-fg { height: 100%; border-radius: 3px; transition: width 0.3s; }
.proto-bar-val { position: absolute; right: 6px; top: 0; font-size: 10px; line-height: 16px; color: #94a3b8; }

/* Timeline */
#timeline { grid-column: 2; }
#timeline-canvas { width: 100%; height: calc(100% - 30px); display: block; }
</style>
</head>
<body>
<div id="header">
  <h1>wiregraph</h1>
  <div class="stats">
    <div>packets <span id="s-pkts">0</span></div>
    <div>bytes <span id="s-bytes">0</span></div>
    <div>hosts <span id="s-hosts">0</span></div>
    <div>edges <span id="s-edges">0</span></div>
    <div>pps <span id="s-pps">0</span></div>
    <div>uptime <span id="s-up">0s</span></div>
  </div>
</div>

<div class="panel" id="hosts">
  <h2>Top Talkers</h2>
  <table>
    <thead><tr>
      <th data-sort="ip">Host</th>
      <th data-sort="total">Traffic</th>
      <th data-sort="packets">Pkts</th>
      <th>Protocols</th>
    </tr></thead>
    <tbody id="host-body"></tbody>
  </table>
</div>

<div class="panel" id="matrix">
  <h2>Connection Matrix</h2>
  <canvas id="matrix-canvas"></canvas>
</div>

<div class="panel" id="protocols">
  <h2>Protocol Breakdown</h2>
  <div id="proto-bars"></div>
</div>

<div class="panel" id="timeline">
  <h2>Activity Timeline</h2>
  <canvas id="timeline-canvas"></canvas>
</div>

<script>
const PROTO_COLORS = {
  HTTP:'#00ffff', TLS:'#00ff88', DNS:'#ffff00', SSH:'#ff8800',
  TCP:'#4488ff', UDP:'#aa44ff', ICMP:'#ff4444', DHCP:'#88ff44',
  NTP:'#ff88ff', SMTP:'#ff6644', OTHER:'#888888',
};

let nodes = [], edgeData = [], selectedIp = null;
let sortKey = 'total', sortDir = -1;
const timeline = []; // rolling window of {ts, count}

// --- Polling ---
async function pollTopology() {
  try {
    const r = await fetch('/api/topology');
    if (r.ok) { const d = await r.json(); nodes = d.nodes; edgeData = d.edges; render(); }
  } catch {}
  setTimeout(pollTopology, 1000);
}

async function pollStats() {
  try {
    const r = await fetch('/api/stats');
    if (r.ok) {
      const s = await r.json();
      document.getElementById('s-pkts').textContent = s.total_packets.toLocaleString();
      document.getElementById('s-bytes').textContent = fmtB(s.total_bytes);
      document.getElementById('s-hosts').textContent = s.host_count;
      document.getElementById('s-edges').textContent = s.edge_count;
      document.getElementById('s-pps').textContent = s.packets_per_second.toFixed(1);
      document.getElementById('s-up').textContent = s.capture_duration.toFixed(0) + 's';
      timeline.push({ ts: Date.now(), pkts: s.total_packets, pps: s.packets_per_second });
      if (timeline.length > 60) timeline.shift();
    }
  } catch {}
  setTimeout(pollStats, 1000);
}

function fmtB(b) {
  if (b < 1024) return b + ' B';
  if (b < 1048576) return (b/1024).toFixed(1) + ' KB';
  if (b < 1073741824) return (b/1048576).toFixed(1) + ' MB';
  return (b/1073741824).toFixed(1) + ' GB';
}

// --- Render all panels ---
function render() {
  renderHosts();
  renderMatrix();
  renderProtocols();
  renderTimeline();
}

// --- Top Talkers ---
function renderHosts() {
  const sorted = [...nodes].sort((a, b) => {
    if (sortKey === 'ip') return sortDir * a.ip.localeCompare(b.ip);
    if (sortKey === 'packets') return sortDir * (a.packet_count - b.packet_count);
    return sortDir * ((a.bytes_sent + a.bytes_recv) - (b.bytes_sent + b.bytes_recv));
  });

  const maxBytes = Math.max(1, ...sorted.map(n => n.bytes_sent + n.bytes_recv));
  const tbody = document.getElementById('host-body');
  tbody.innerHTML = '';

  for (const n of sorted) {
    const total = n.bytes_sent + n.bytes_recv;
    const pct = total / maxBytes * 100;
    const cls = n.is_local ? 'local' : 'remote';
    const sel = n.ip === selectedIp ? ' selected' : '';
    const protos = (n.protocols || []).map(p => {
      const c = PROTO_COLORS[p] || PROTO_COLORS.OTHER;
      return `<span class="proto-tag" style="background:${c}22;color:${c}">${p}</span>`;
    }).join('');

    const tr = document.createElement('tr');
    tr.className = sel.trim();
    tr.innerHTML = `
      <td><span class="ip ${cls}">${n.ip}</span><br><span style="font-size:10px;color:#475569">${n.subnet}</span></td>
      <td class="bar-cell">
        <div class="bar-fill" style="width:${pct}%;background:${n.is_local ? '#34d399' : '#64748b'}"></div>
        <span class="bar-text">${fmtB(total)}</span>
      </td>
      <td>${n.packet_count.toLocaleString()}</td>
      <td>${protos}</td>`;
    tr.addEventListener('click', () => { selectedIp = (selectedIp === n.ip) ? null : n.ip; render(); });
    tbody.appendChild(tr);
  }
}

// Column sort
document.querySelectorAll('#hosts th[data-sort]').forEach(th => {
  th.addEventListener('click', () => {
    const key = th.dataset.sort;
    if (sortKey === key) sortDir *= -1;
    else { sortKey = key; sortDir = -1; }
    renderHosts();
  });
});

// --- Connection Matrix ---
function renderMatrix() {
  const canvas = document.getElementById('matrix-canvas');
  const rect = canvas.parentElement.getBoundingClientRect();
  const w = rect.width - 24;
  const h = rect.height - 40;
  canvas.width = w * devicePixelRatio;
  canvas.height = h * devicePixelRatio;
  canvas.style.width = w + 'px';
  canvas.style.height = h + 'px';
  const ctx = canvas.getContext('2d');
  ctx.scale(devicePixelRatio, devicePixelRatio);

  const ips = nodes.map(n => n.ip).sort();
  const n = ips.length;
  if (n === 0) { ctx.fillStyle = '#475569'; ctx.font = '12px monospace'; ctx.fillText('waiting for data...', 20, 30); return; }

  const ipIdx = {};
  ips.forEach((ip, i) => ipIdx[ip] = i);

  // Build matrix
  const matrix = Array.from({length: n}, () => Array(n).fill(0));
  const protoMatrix = Array.from({length: n}, () => Array(n).fill(''));
  let maxVal = 1;
  for (const e of edgeData) {
    const si = ipIdx[e.source], ti = ipIdx[e.target];
    if (si !== undefined && ti !== undefined) {
      matrix[si][ti] += e.bytes;
      matrix[ti][si] += e.bytes;
      protoMatrix[si][ti] = e.protocol;
      protoMatrix[ti][si] = e.protocol;
      maxVal = Math.max(maxVal, matrix[si][ti]);
    }
  }

  // Layout
  const labelW = Math.min(100, w * 0.25);
  const cellSize = Math.min(Math.floor((w - labelW) / n), Math.floor((h - labelW) / n), 40);
  const gridW = cellSize * n;

  ctx.clearRect(0, 0, w, h);

  // Row labels (left)
  ctx.font = '10px monospace';
  ctx.textAlign = 'right';
  ctx.textBaseline = 'middle';
  for (let i = 0; i < n; i++) {
    const node = nodes.find(nd => nd.ip === ips[i]);
    ctx.fillStyle = node && node.is_local ? '#34d399' : '#64748b';
    ctx.fillText(ips[i], labelW - 4, labelW + i * cellSize + cellSize / 2);
  }

  // Column labels (top, rotated)
  ctx.save();
  ctx.textAlign = 'left';
  for (let j = 0; j < n; j++) {
    const node = nodes.find(nd => nd.ip === ips[j]);
    ctx.fillStyle = node && node.is_local ? '#34d399' : '#64748b';
    ctx.save();
    ctx.translate(labelW + j * cellSize + cellSize / 2, labelW - 4);
    ctx.rotate(-Math.PI / 4);
    ctx.fillText(ips[j], 0, 0);
    ctx.restore();
  }
  ctx.restore();

  // Cells
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      const x = labelW + j * cellSize;
      const y = labelW + i * cellSize;
      const val = matrix[i][j];

      if (val > 0) {
        const intensity = Math.log2(val + 1) / Math.log2(maxVal + 1);
        const proto = protoMatrix[i][j];
        const baseColor = PROTO_COLORS[proto] || PROTO_COLORS.OTHER;
        ctx.fillStyle = hexAlpha(baseColor, 0.15 + intensity * 0.75);
        ctx.fillRect(x + 1, y + 1, cellSize - 2, cellSize - 2);

        if (cellSize > 20) {
          ctx.fillStyle = '#c0ccdd';
          ctx.font = '9px monospace';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText(fmtBShort(val), x + cellSize/2, y + cellSize/2);
        }
      } else {
        ctx.fillStyle = '#0f172a';
        ctx.fillRect(x + 1, y + 1, cellSize - 2, cellSize - 2);
      }

      // Highlight selected
      if (selectedIp && (ips[i] === selectedIp || ips[j] === selectedIp)) {
        ctx.strokeStyle = '#7dd3fc44';
        ctx.strokeRect(x, y, cellSize, cellSize);
      }
    }
  }
}

function hexAlpha(hex, a) {
  const r = parseInt(hex.slice(1,3), 16);
  const g = parseInt(hex.slice(3,5), 16);
  const b = parseInt(hex.slice(5,7), 16);
  return `rgba(${r},${g},${b},${a.toFixed(2)})`;
}

function fmtBShort(b) {
  if (b < 1024) return b + '';
  if (b < 1048576) return (b/1024).toFixed(0) + 'K';
  return (b/1048576).toFixed(0) + 'M';
}

// --- Protocol Breakdown ---
function renderProtocols() {
  const counts = {};
  for (const e of edgeData) {
    counts[e.protocol] = (counts[e.protocol] || 0) + e.bytes;
  }

  const sorted = Object.entries(counts).sort((a, b) => b[1] - a[1]);
  const maxVal = Math.max(1, ...sorted.map(s => s[1]));
  const container = document.getElementById('proto-bars');
  container.innerHTML = '';

  for (const [proto, bytes] of sorted) {
    const pct = bytes / maxVal * 100;
    const color = PROTO_COLORS[proto] || PROTO_COLORS.OTHER;
    container.innerHTML += `
      <div class="proto-row">
        <div class="proto-label" style="color:${color}">${proto}</div>
        <div class="proto-bar-bg">
          <div class="proto-bar-fg" style="width:${pct}%;background:${color}"></div>
          <span class="proto-bar-val">${fmtB(bytes)}</span>
        </div>
      </div>`;
  }
}

// --- Activity Timeline ---
function renderTimeline() {
  const canvas = document.getElementById('timeline-canvas');
  const rect = canvas.parentElement.getBoundingClientRect();
  const w = rect.width - 24;
  const h = rect.height - 40;
  canvas.width = w * devicePixelRatio;
  canvas.height = h * devicePixelRatio;
  canvas.style.width = w + 'px';
  canvas.style.height = h + 'px';
  const ctx = canvas.getContext('2d');
  ctx.scale(devicePixelRatio, devicePixelRatio);
  ctx.clearRect(0, 0, w, h);

  if (timeline.length < 2) {
    ctx.fillStyle = '#475569'; ctx.font = '12px monospace';
    ctx.fillText('collecting data...', 20, 30);
    return;
  }

  const margin = { top: 10, right: 10, bottom: 20, left: 40 };
  const pw = w - margin.left - margin.right;
  const ph = h - margin.top - margin.bottom;

  const maxPps = Math.max(1, ...timeline.map(t => t.pps));

  // Grid lines
  ctx.strokeStyle = '#1e293b';
  ctx.lineWidth = 1;
  for (let i = 0; i <= 4; i++) {
    const y = margin.top + ph * (1 - i / 4);
    ctx.beginPath(); ctx.moveTo(margin.left, y); ctx.lineTo(w - margin.right, y); ctx.stroke();
    ctx.fillStyle = '#475569'; ctx.font = '10px monospace'; ctx.textAlign = 'right';
    ctx.fillText((maxPps * i / 4).toFixed(0), margin.left - 4, y + 3);
  }

  // Axis labels
  ctx.fillStyle = '#475569'; ctx.font = '10px monospace'; ctx.textAlign = 'center';
  ctx.fillText('pps', margin.left - 4, margin.top - 2);

  // Line
  ctx.beginPath();
  ctx.strokeStyle = '#7dd3fc';
  ctx.lineWidth = 2;
  for (let i = 0; i < timeline.length; i++) {
    const x = margin.left + (i / (timeline.length - 1)) * pw;
    const y = margin.top + ph * (1 - timeline[i].pps / maxPps);
    if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
  }
  ctx.stroke();

  // Fill under line
  ctx.lineTo(margin.left + pw, margin.top + ph);
  ctx.lineTo(margin.left, margin.top + ph);
  ctx.closePath();
  ctx.fillStyle = 'rgba(125, 211, 252, 0.08)';
  ctx.fill();
}

// --- Matrix click ---
document.getElementById('matrix-canvas').addEventListener('click', (e) => {
  const canvas = e.target;
  const rect = canvas.getBoundingClientRect();
  const x = (e.clientX - rect.left);
  const ips = nodes.map(n => n.ip).sort();
  const n = ips.length;
  if (n === 0) return;
  const labelW = Math.min(100, rect.width * 0.25);
  const cellSize = Math.min(Math.floor((rect.width - labelW) / n), 40);
  const col = Math.floor((x - labelW) / cellSize);
  if (col >= 0 && col < n) {
    selectedIp = (selectedIp === ips[col]) ? null : ips[col];
    render();
  }
});

// --- Start ---
pollTopology();
pollStats();
setInterval(renderTimeline, 1000);
</script>
</body>
</html>
"##;
