pub const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>wiregraph</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  background: #0b0e17; color: #c0ccdd; font: 13px/1.5 'JetBrains Mono', 'Fira Code', monospace;
  display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: auto 1fr 1fr auto;
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

/* Export controls */
.export-bar {
  display: flex; align-items: center; gap: 8px;
}
.export-bar button {
  background: #1e3a5f; color: #7dd3fc; border: 1px solid #2563eb; border-radius: 4px;
  padding: 4px 12px; font: 12px monospace; cursor: pointer;
}
.export-bar button:hover { background: #2563eb; color: #fff; }
.export-bar .hint { font-size: 11px; color: #475569; }
#retention-info { font-size: 10px; color: #475569; margin-left: 8px; }
#retention-bar {
  display: inline-block; width: 40px; height: 6px; background: #1e293b;
  border-radius: 3px; vertical-align: middle; margin: 0 4px; overflow: hidden;
}
#retention-fill {
  height: 100%; border-radius: 3px; transition: width 0.5s, background 0.5s;
}

/* Search input */
#search-input {
  background: #0b0e17; color: #e2e8f0; border: 1px solid #1e293b; border-radius: 4px;
  padding: 4px 10px; font: 12px monospace; width: 200px; outline: none;
}
#search-input:focus { border-color: #2563eb; }
#search-input::placeholder { color: #475569; }

/* Protocol toggle states */
.proto-row { cursor: pointer; user-select: none; transition: opacity 0.2s; }
.proto-row.disabled { opacity: 0.3; }

/* Drawer */
#drawer {
  grid-column: 1 / -1; display: none;
  background: #111827; border: 1px solid #1e293b; border-radius: 6px;
  padding: 12px; max-height: 40vh; overflow: auto;
}
#drawer.open { display: block; }
#drawer-header {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 8px; border-bottom: 1px solid #1e293b; padding-bottom: 6px;
}
#drawer-header h2 { color: #7dd3fc; font-size: 12px; text-transform: uppercase; letter-spacing: 1px; margin: 0; }
#drawer-close {
  background: none; border: none; color: #64748b; font-size: 16px; cursor: pointer;
}
#drawer-close:hover { color: #e2e8f0; }
#drawer-tabs {
  display: flex; gap: 0; margin-bottom: 8px;
}
#drawer-tabs button {
  background: #0b0e17; color: #64748b; border: 1px solid #1e293b; padding: 4px 14px;
  font: 12px monospace; cursor: pointer;
}
#drawer-tabs button:first-child { border-radius: 4px 0 0 4px; }
#drawer-tabs button:last-child { border-radius: 0 4px 4px 0; }
#drawer-tabs button.active { background: #1e3a5f; color: #7dd3fc; border-color: #2563eb; }

/* Packet table in drawer */
#packet-table { width: 100%; border-collapse: collapse; font-size: 12px; }
#packet-table th {
  text-align: left; color: #64748b; font-weight: normal; padding: 4px 8px;
  border-bottom: 1px solid #1e293b; cursor: pointer; user-select: none;
}
#packet-table th:hover { color: #7dd3fc; }
#packet-table td { padding: 4px 8px; border-bottom: 1px solid #0f172a; }
#packet-table tr:hover td { background: #1e293b; }
#pkt-pagination {
  display: flex; align-items: center; justify-content: space-between;
  margin-top: 8px; font-size: 11px; color: #64748b;
}
#pkt-pagination button {
  background: #1e3a5f; color: #7dd3fc; border: 1px solid #2563eb; border-radius: 4px;
  padding: 2px 10px; font: 11px monospace; cursor: pointer;
}
#pkt-pagination button:disabled { opacity: 0.4; cursor: default; }

/* Conversation view */
#conv-view { display: none; }
#conv-view.active { display: block; }
#conv-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.conv-col h3 { color: #7dd3fc; font-size: 12px; margin-bottom: 8px; }
.conv-stat { display: flex; justify-content: space-between; padding: 3px 0; font-size: 12px; border-bottom: 1px solid #0f172a; }
.conv-stat .label { color: #64748b; }
.conv-stat .val { color: #e2e8f0; }
.conv-proto-bars { margin-top: 8px; }
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
  <input id="search-input" type="text" placeholder="IP, CIDR, port:N" spellcheck="false">
  <div class="export-bar">
    <button id="export-all" title="Download all captured packets">Export pcap</button>
    <button id="export-selected" title="Download only packets matching selected host" style="display:none">Export selected</button>
    <span class="hint" id="export-hint"></span>
    <span id="retention-info"><span id="retention-bar"><span id="retention-fill"></span></span><span id="retention-text"></span></span>
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

<div id="drawer">
  <div id="drawer-header">
    <h2 id="drawer-title">Packets</h2>
    <div id="drawer-tabs">
      <button class="active" data-tab="packets">Packets</button>
      <button data-tab="conversation">Conversation</button>
    </div>
    <button id="drawer-close">&times;</button>
  </div>
  <div id="packets-view">
    <table id="packet-table">
      <thead><tr>
        <th data-col="timestamp">Time</th>
        <th data-col="src_ip">Source</th>
        <th data-col="dst_ip">Destination</th>
        <th data-col="protocol">Protocol</th>
        <th data-col="dst_port">Port</th>
        <th data-col="size">Size</th>
      </tr></thead>
      <tbody id="pkt-body"></tbody>
    </table>
    <div id="pkt-pagination">
      <span id="pkt-info"></span>
      <div>
        <button id="pkt-prev">Prev</button>
        <button id="pkt-next">Next</button>
      </div>
    </div>
  </div>
  <div id="conv-view">
    <div id="conv-columns">
      <div class="conv-col" id="conv-a"></div>
      <div class="conv-col" id="conv-b"></div>
    </div>
  </div>
</div>

<script>
const PROTO_COLORS = {
  HTTP:'#00ffff', TLS:'#00ff88', DNS:'#ffff00', SSH:'#ff8800',
  TCP:'#4488ff', UDP:'#aa44ff', ICMP:'#ff4444', DHCP:'#88ff44',
  NTP:'#ff88ff', SMTP:'#ff6644', OTHER:'#888888',
};

let nodes = [], edgeData = [], selectedIp = null;
let sortKey = 'total', sortDir = -1;
const timeline = []; // rolling window of {ts, protos: {TCP: bps, UDP: bps, ...}}
let prevProtoTotals = null; // previous snapshot for delta computation
let prevProtoTs = null;

// --- Filter state ---
let enabledProtocols = new Set(); // empty = all enabled
let searchFilter = { type: null, value: null }; // {type: 'ip'|'cidr'|'port'|'string', value: ...}
let conversationPair = null; // {a, b} or null
let drawerTab = 'packets';
let pktOffset = 0, pktLimit = 100, pktTotal = 0;
let pktSortCol = 'timestamp', pktSortDir = 1;

function ipInSubnet(ip, cidr) {
  const [subnet, bits] = cidr.split('/');
  const mask = ~0 << (32 - parseInt(bits));
  const ipParts = ip.split('.').map(Number);
  const subParts = subnet.split('.').map(Number);
  const ipNum = (ipParts[0]<<24)|(ipParts[1]<<16)|(ipParts[2]<<8)|ipParts[3];
  const subNum = (subParts[0]<<24)|(subParts[1]<<16)|(subParts[2]<<8)|subParts[3];
  return (ipNum & mask) === (subNum & mask);
}

function parseSearch(val) {
  val = val.trim();
  if (!val) return { type: null, value: null };
  if (/^port:(\d+)$/i.test(val)) return { type: 'port', value: parseInt(val.split(':')[1]) };
  if (/^\d+\.\d+\.\d+\.\d+\/\d+$/.test(val)) return { type: 'cidr', value: val };
  if (/^\d+\.\d+\.\d+\.\d+$/.test(val)) return { type: 'ip', value: val };
  return { type: 'string', value: val.toLowerCase() };
}

function matchesFilters(node) {
  // Protocol filter
  if (enabledProtocols.size > 0) {
    const protos = node.protocols || [];
    if (!protos.some(p => enabledProtocols.has(p))) return false;
  }
  // Search filter
  if (searchFilter.type === 'ip') {
    if (node.ip !== searchFilter.value) return false;
  } else if (searchFilter.type === 'cidr') {
    if (!ipInSubnet(node.ip, searchFilter.value)) return false;
  } else if (searchFilter.type === 'port') {
    // Port filter can't be checked on node level, allow through
  } else if (searchFilter.type === 'string') {
    if (!node.ip.toLowerCase().includes(searchFilter.value) &&
        !(node.subnet || '').toLowerCase().includes(searchFilter.value) &&
        !(node.protocols || []).some(p => p.toLowerCase().includes(searchFilter.value))) return false;
  }
  return true;
}

function matchesEdgeFilters(edge) {
  if (enabledProtocols.size > 0 && !enabledProtocols.has(edge.protocol)) return false;
  if (searchFilter.type === 'ip') {
    if (edge.source !== searchFilter.value && edge.target !== searchFilter.value) return false;
  } else if (searchFilter.type === 'cidr') {
    if (!ipInSubnet(edge.source, searchFilter.value) && !ipInSubnet(edge.target, searchFilter.value)) return false;
  } else if (searchFilter.type === 'port') {
    // Edge-level port not available, allow through
  } else if (searchFilter.type === 'string') {
    const v = searchFilter.value;
    if (!edge.source.includes(v) && !edge.target.includes(v) && !edge.protocol.toLowerCase().includes(v)) return false;
  }
  return true;
}

function filteredNodes() { return nodes.filter(matchesFilters); }
function filteredEdges() { return edgeData.filter(matchesEdgeFilters); }

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
  snapshotTimeline();
  renderTimeline();
  if (typeof updateExportButtons === 'function') updateExportButtons();
}

function snapshotTimeline() {
  const fEdges = filteredEdges();
  const totals = {};
  for (const e of fEdges) {
    totals[e.protocol] = (totals[e.protocol] || 0) + e.bytes;
  }
  const now = Date.now();
  if (prevProtoTotals && prevProtoTs) {
    const dt = (now - prevProtoTs) / 1000;
    if (dt > 0.1) {
      const protos = {};
      const allKeys = new Set([...Object.keys(totals), ...Object.keys(prevProtoTotals)]);
      for (const k of allKeys) {
        const delta = (totals[k] || 0) - (prevProtoTotals[k] || 0);
        if (delta > 0) protos[k] = delta / dt;
      }
      timeline.push({ ts: now, protos });
      if (timeline.length > 120) timeline.shift();
    }
  }
  prevProtoTotals = totals;
  prevProtoTs = now;
}

// --- Top Talkers ---
function renderHosts() {
  const sorted = [...filteredNodes()].sort((a, b) => {
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
    tr.addEventListener('click', (e) => {
      if (e.shiftKey && selectedIp && selectedIp !== n.ip) {
        // Shift+click: conversation between selected and this host
        conversationPair = { a: selectedIp, b: n.ip };
        drawerTab = 'conversation';
        openDrawer();
        fetchConversation();
      } else {
        selectedIp = (selectedIp === n.ip) ? null : n.ip;
        conversationPair = null;
        render();
        maybeOpenDrawer();
      }
    });
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

  const fNodes = filteredNodes();
  const fEdges = filteredEdges();
  const ips = fNodes.map(n => n.ip).sort();
  const n = ips.length;
  if (n === 0) { ctx.fillStyle = '#475569'; ctx.font = '12px monospace'; ctx.fillText('waiting for data...', 20, 30); return; }

  const ipIdx = {};
  ips.forEach((ip, i) => ipIdx[ip] = i);

  // Build matrix
  const matrix = Array.from({length: n}, () => Array(n).fill(0));
  const protoMatrix = Array.from({length: n}, () => Array(n).fill(''));
  let maxVal = 1;
  for (const e of fEdges) {
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
  // Always count from unfiltered edges for protocol bars (so you can see what's available)
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
    const disabled = enabledProtocols.size > 0 && !enabledProtocols.has(proto);
    const row = document.createElement('div');
    row.className = 'proto-row' + (disabled ? ' disabled' : '');
    row.innerHTML = `
        <div class="proto-label" style="color:${color}">${proto}</div>
        <div class="proto-bar-bg">
          <div class="proto-bar-fg" style="width:${pct}%;background:${color}"></div>
          <span class="proto-bar-val">${fmtB(bytes)}</span>
        </div>`;
    row.addEventListener('click', () => {
      if (enabledProtocols.size === 1 && enabledProtocols.has(proto)) {
        enabledProtocols.clear(); // toggle off = show all
      } else {
        enabledProtocols.clear();
        enabledProtocols.add(proto); // show only this protocol
      }
      render();
      maybeOpenDrawer();
    });
    container.appendChild(row);
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

  const margin = { top: 10, right: 10, bottom: 20, left: 50 };
  const pw = w - margin.left - margin.right;
  const ph = h - margin.top - margin.bottom;

  // Collect all protocols seen across timeline
  const allProtos = new Set();
  for (const t of timeline) {
    for (const p of Object.keys(t.protos)) allProtos.add(p);
  }
  // Sort by total bytes desc for consistent stacking
  const protoOrder = [...allProtos].sort((a, b) => {
    const aSum = timeline.reduce((s, t) => s + (t.protos[a] || 0), 0);
    const bSum = timeline.reduce((s, t) => s + (t.protos[b] || 0), 0);
    return bSum - aSum;
  });

  if (protoOrder.length === 0) {
    ctx.fillStyle = '#475569'; ctx.font = '12px monospace';
    ctx.fillText('no activity in window', 20, 30);
    return;
  }

  // Compute stacked values and find max
  const stacked = timeline.map(t => {
    let cumulative = 0;
    const layers = {};
    for (const p of protoOrder) {
      cumulative += (t.protos[p] || 0);
      layers[p] = cumulative;
    }
    return { total: cumulative, layers };
  });
  const maxBps = Math.max(1, ...stacked.map(s => s.total));

  // Grid lines
  ctx.strokeStyle = '#1e293b';
  ctx.lineWidth = 1;
  for (let i = 0; i <= 4; i++) {
    const y = margin.top + ph * (1 - i / 4);
    ctx.beginPath(); ctx.moveTo(margin.left, y); ctx.lineTo(w - margin.right, y); ctx.stroke();
    ctx.fillStyle = '#475569'; ctx.font = '10px monospace'; ctx.textAlign = 'right';
    ctx.fillText(fmtBps(maxBps * i / 4), margin.left - 4, y + 3);
  }

  // Draw stacked areas (back to front, largest first)
  const baseline = margin.top + ph;
  for (let pi = protoOrder.length - 1; pi >= 0; pi--) {
    const proto = protoOrder[pi];
    const color = PROTO_COLORS[proto] || PROTO_COLORS.OTHER;

    ctx.beginPath();
    for (let i = 0; i < timeline.length; i++) {
      const x = margin.left + (i / (timeline.length - 1)) * pw;
      const y = baseline - (stacked[i].layers[proto] / maxBps) * ph;
      if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    }
    // Close along the bottom of this layer (top of the layer below, or baseline)
    if (pi > 0) {
      const belowProto = protoOrder[pi - 1];
      for (let i = timeline.length - 1; i >= 0; i--) {
        const x = margin.left + (i / (timeline.length - 1)) * pw;
        const y = baseline - (stacked[i].layers[belowProto] / maxBps) * ph;
        ctx.lineTo(x, y);
      }
    } else {
      ctx.lineTo(margin.left + pw, baseline);
      ctx.lineTo(margin.left, baseline);
    }
    ctx.closePath();
    ctx.fillStyle = hexAlpha(color, 0.35);
    ctx.fill();

    // Stroke the top edge
    ctx.beginPath();
    for (let i = 0; i < timeline.length; i++) {
      const x = margin.left + (i / (timeline.length - 1)) * pw;
      const y = baseline - (stacked[i].layers[proto] / maxBps) * ph;
      if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    }
    ctx.strokeStyle = hexAlpha(color, 0.7);
    ctx.lineWidth = 1;
    ctx.stroke();
  }

  // Legend (top-right, compact)
  const legendX = w - margin.right - 8;
  let legendY = margin.top + 4;
  ctx.textAlign = 'right';
  ctx.font = '9px monospace';
  for (const proto of protoOrder.slice(0, 6)) {
    const color = PROTO_COLORS[proto] || PROTO_COLORS.OTHER;
    ctx.fillStyle = hexAlpha(color, 0.6);
    ctx.fillRect(legendX + 2, legendY - 5, 8, 8);
    ctx.fillStyle = '#94a3b8';
    ctx.fillText(proto, legendX - 2, legendY + 2);
    legendY += 12;
  }

  // Time labels along bottom
  ctx.fillStyle = '#475569'; ctx.font = '9px monospace'; ctx.textAlign = 'center';
  const now = timeline[timeline.length - 1].ts;
  for (let i = 0; i < 5; i++) {
    const idx = Math.floor(i / 4 * (timeline.length - 1));
    const secsAgo = Math.round((now - timeline[idx].ts) / 1000);
    const x = margin.left + (idx / (timeline.length - 1)) * pw;
    ctx.fillText(secsAgo > 0 ? `-${secsAgo}s` : 'now', x, baseline + 14);
  }
}

function fmtBps(bps) {
  if (bps < 1024) return bps.toFixed(0) + ' B/s';
  if (bps < 1048576) return (bps/1024).toFixed(1) + ' KB/s';
  return (bps/1048576).toFixed(1) + ' MB/s';
}

// --- Matrix click ---
document.getElementById('matrix-canvas').addEventListener('click', (e) => {
  const canvas = e.target;
  const rect = canvas.getBoundingClientRect();
  const x = (e.clientX - rect.left);
  const y = (e.clientY - rect.top);
  const fNodes = filteredNodes();
  const ips = fNodes.map(n => n.ip).sort();
  const n = ips.length;
  if (n === 0) return;
  const labelW = Math.min(100, rect.width * 0.25);
  const cellSize = Math.min(Math.floor((rect.width - labelW) / n), Math.floor((rect.height - labelW) / n), 40);
  const col = Math.floor((x - labelW) / cellSize);
  const row = Math.floor((y - labelW) / cellSize);
  if (col >= 0 && col < n && row >= 0 && row < n) {
    if (row !== col) {
      conversationPair = { a: ips[row], b: ips[col] };
      drawerTab = 'conversation';
      openDrawer();
      fetchConversation();
    } else {
      selectedIp = (selectedIp === ips[col]) ? null : ips[col];
      conversationPair = null;
      render();
      maybeOpenDrawer();
    }
  }
});

// --- Export ---
function exportPcap(filter) {
  const params = new URLSearchParams();
  if (filter.hosts && filter.hosts.length) params.set('hosts', filter.hosts.join(','));
  // Merge explicit protocols with toggle state
  const protos = filter.protocols && filter.protocols.length
    ? filter.protocols
    : enabledProtocols.size > 0 ? [...enabledProtocols] : [];
  if (protos.length) params.set('protocols', protos.join(','));
  const url = '/api/export' + (params.toString() ? '?' + params : '');
  const a = document.createElement('a');
  a.href = url;
  a.download = 'wiregraph-export.pcap';
  a.click();
}

document.getElementById('export-all').addEventListener('click', () => exportPcap({}));
document.getElementById('export-selected').addEventListener('click', () => {
  if (selectedIp) exportPcap({ hosts: [selectedIp] });
});

function updateExportButtons() {
  const selBtn = document.getElementById('export-selected');
  const hint = document.getElementById('export-hint');
  if (selectedIp) {
    selBtn.style.display = '';
    selBtn.textContent = 'Export ' + selectedIp;
    hint.textContent = '';
  } else {
    selBtn.style.display = 'none';
    hint.textContent = 'click a host to filter export';
  }
}

// --- Search ---
let searchTimeout = null;
document.getElementById('search-input').addEventListener('input', (e) => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    searchFilter = parseSearch(e.target.value);
    render();
    maybeOpenDrawer();
  }, 150);
});

// --- Drawer ---
function openDrawer() {
  document.getElementById('drawer').classList.add('open');
  const title = document.getElementById('drawer-title');
  if (conversationPair) {
    title.textContent = conversationPair.a + ' \u2194 ' + conversationPair.b;
  } else if (selectedIp) {
    title.textContent = selectedIp + '  (shift+click another host for conversation)';
  } else {
    title.textContent = 'Packets';
  }
  updateDrawerTabs();
}

function closeDrawer() {
  document.getElementById('drawer').classList.remove('open');
  conversationPair = null;
}

function maybeOpenDrawer() {
  if (selectedIp || searchFilter.type) {
    drawerTab = 'packets';
    openDrawer();
    pktOffset = 0;
    fetchPackets();
  } else {
    closeDrawer();
  }
}

document.getElementById('drawer-close').addEventListener('click', () => {
  selectedIp = null;
  searchFilter = { type: null, value: null };
  document.getElementById('search-input').value = '';
  conversationPair = null;
  closeDrawer();
  render();
});

document.querySelectorAll('#drawer-tabs button').forEach(btn => {
  btn.addEventListener('click', () => {
    drawerTab = btn.dataset.tab;
    updateDrawerTabs();
  });
});

function updateDrawerTabs() {
  const convBtn = document.querySelector('#drawer-tabs button[data-tab="conversation"]');
  if (convBtn) {
    convBtn.disabled = !conversationPair;
    convBtn.style.opacity = conversationPair ? '' : '0.3';
    convBtn.style.cursor = conversationPair ? 'pointer' : 'default';
  }
  if (drawerTab === 'conversation' && !conversationPair) drawerTab = 'packets';
  document.querySelectorAll('#drawer-tabs button').forEach(btn => {
    btn.classList.toggle('active', btn.dataset.tab === drawerTab);
  });
  document.getElementById('packets-view').style.display = drawerTab === 'packets' ? '' : 'none';
  document.getElementById('conv-view').className = drawerTab === 'conversation' ? 'active' : '';
  if (drawerTab === 'packets') fetchPackets();
}

// --- Packet table ---
function buildPacketParams() {
  const params = new URLSearchParams();
  if (selectedIp) params.set('hosts', selectedIp);
  else if (searchFilter.type === 'ip') params.set('hosts', searchFilter.value);
  if (enabledProtocols.size > 0) params.set('protocols', [...enabledProtocols].join(','));
  if (searchFilter.type === 'port') params.set('port', searchFilter.value);
  params.set('limit', pktLimit);
  params.set('offset', pktOffset);
  return params;
}

async function fetchPackets() {
  try {
    const params = buildPacketParams();
    const r = await fetch('/api/packets?' + params);
    if (!r.ok) return;
    const page = await r.json();
    pktTotal = page.total;
    renderPacketTable(page.packets);
    renderPagination();
  } catch {}
}

function renderPacketTable(packets) {
  const tbody = document.getElementById('pkt-body');
  tbody.innerHTML = '';
  for (const p of packets) {
    const t = new Date(p.timestamp * 1000);
    const ts = t.toLocaleTimeString() + '.' + String(t.getMilliseconds()).padStart(3, '0');
    const color = PROTO_COLORS[p.protocol] || PROTO_COLORS.OTHER;
    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td style="color:#64748b">${ts}</td>
      <td class="ip">${p.src_ip}</td>
      <td class="ip">${p.dst_ip}</td>
      <td><span class="proto-tag" style="background:${color}22;color:${color}">${p.protocol}</span></td>
      <td>${p.dst_port}</td>
      <td>${fmtB(p.size)}</td>`;
    tbody.appendChild(tr);
  }
}

function renderPagination() {
  const start = pktTotal === 0 ? 0 : pktOffset + 1;
  const end = Math.min(pktOffset + pktLimit, pktTotal);
  document.getElementById('pkt-info').textContent = `showing ${start}-${end} of ${pktTotal}`;
  document.getElementById('pkt-prev').disabled = pktOffset === 0;
  document.getElementById('pkt-next').disabled = pktOffset + pktLimit >= pktTotal;
}

document.getElementById('pkt-prev').addEventListener('click', () => {
  pktOffset = Math.max(0, pktOffset - pktLimit);
  fetchPackets();
});
document.getElementById('pkt-next').addEventListener('click', () => {
  pktOffset += pktLimit;
  fetchPackets();
});

// Sortable packet table columns
document.querySelectorAll('#packet-table th[data-col]').forEach(th => {
  th.addEventListener('click', () => {
    // Client-side sort not available (server-side pagination), but we toggle for visual feedback
    // Future: add sort param to API
  });
});

// --- Conversation view ---
async function fetchConversation() {
  if (!conversationPair) return;
  try {
    const params = new URLSearchParams({ a: conversationPair.a, b: conversationPair.b });
    const r = await fetch('/api/conversation?' + params);
    if (!r.ok) return;
    const conv = await r.json();
    renderConversation(conv);
    openDrawer();
  } catch {}
}

function renderConversation(conv) {
  const totalBytes = conv.a_to_b_bytes + conv.b_to_a_bytes;
  const totalPkts = conv.a_to_b_packets + conv.b_to_a_packets;

  document.getElementById('conv-a').innerHTML = `
    <h3>${conv.host_a} &rarr; ${conv.host_b}</h3>
    <div class="conv-stat"><span class="label">Bytes</span><span class="val">${fmtB(conv.a_to_b_bytes)}</span></div>
    <div class="conv-stat"><span class="label">Packets</span><span class="val">${conv.a_to_b_packets.toLocaleString()}</span></div>
    <div class="conv-stat"><span class="label">Share</span><span class="val">${totalBytes ? (conv.a_to_b_bytes/totalBytes*100).toFixed(1) : 0}%</span></div>
  `;

  document.getElementById('conv-b').innerHTML = `
    <h3>${conv.host_b} &rarr; ${conv.host_a}</h3>
    <div class="conv-stat"><span class="label">Bytes</span><span class="val">${fmtB(conv.b_to_a_bytes)}</span></div>
    <div class="conv-stat"><span class="label">Packets</span><span class="val">${conv.b_to_a_packets.toLocaleString()}</span></div>
    <div class="conv-stat"><span class="label">Share</span><span class="val">${totalBytes ? (conv.b_to_a_bytes/totalBytes*100).toFixed(1) : 0}%</span></div>
  `;

  // Protocol breakdown bars
  const protos = Object.entries(conv.protocols).sort((a,b) => b[1]-a[1]);
  const maxP = Math.max(1, ...protos.map(p => p[1]));
  let barsHtml = '<div class="conv-proto-bars">';
  for (const [proto, bytes] of protos) {
    const color = PROTO_COLORS[proto] || PROTO_COLORS.OTHER;
    const pct = bytes / maxP * 100;
    barsHtml += `<div class="proto-row">
      <div class="proto-label" style="color:${color}">${proto}</div>
      <div class="proto-bar-bg">
        <div class="proto-bar-fg" style="width:${pct}%;background:${color}"></div>
        <span class="proto-bar-val">${fmtB(bytes)}</span>
      </div>
    </div>`;
  }
  barsHtml += '</div>';

  // Timing info
  const dur = conv.duration > 0 ? conv.duration.toFixed(1) + 's' : 'instant';
  document.getElementById('conv-b').innerHTML += `
    <div class="conv-stat" style="margin-top:12px"><span class="label">Duration</span><span class="val">${dur}</span></div>
    <div class="conv-stat"><span class="label">Total</span><span class="val">${fmtB(totalBytes)} / ${totalPkts.toLocaleString()} pkts</span></div>
    ${barsHtml}
  `;
}

// Host row click -> also open drawer
const origHostClick = null; // handled inline in renderHosts

// --- Retention ---
async function pollRetention() {
  try {
    const r = await fetch('/api/retention');
    if (r.ok) {
      const ri = await r.json();
      const fill = document.getElementById('retention-fill');
      const text = document.getElementById('retention-text');
      const pct = Math.min(100, ri.utilization_pct);
      fill.style.width = pct + '%';
      fill.style.background = pct > 90 ? '#ef4444' : pct > 70 ? '#f59e0b' : '#34d399';
      let windowStr = '';
      if (ri.window_secs > 0) {
        if (ri.window_secs >= 3600) windowStr = (ri.window_secs/3600).toFixed(1) + 'h';
        else if (ri.window_secs >= 60) windowStr = (ri.window_secs/60).toFixed(0) + 'm';
        else windowStr = ri.window_secs.toFixed(0) + 's';
        windowStr = 'last ' + windowStr;
      }
      const stored = fmtB(ri.stored_bytes) + ' / ' + fmtB(ri.max_bytes);
      text.textContent = windowStr ? windowStr + ' (' + stored + ')' : stored;
      if (ri.evicted_packets > 0) {
        text.title = ri.evicted_packets.toLocaleString() + ' packets evicted';
      }
    }
  } catch {}
  setTimeout(pollRetention, 2000);
}

// --- Start ---
pollTopology();
pollStats();
pollRetention();
</script>
</body>
</html>
"##;
