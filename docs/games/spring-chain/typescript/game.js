// game.ts
var N = 3800;
var k = 0.18;
var dt = 0.12;
var nodes = [];
for (let i = 0; i < N; i++) {
  nodes.push({
    x: Math.sin(i * 0.018) * 24 + (Math.random() - 0.5) * 2,
    v: 0
  });
}
var canvas = document.getElementById("game-canvas");
var ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");
var elFps = document.getElementById("fps");
var elMs = document.getElementById("compute-ms");
var last = performance.now();
var frames = 0;
var fpsSmooth = 60;
var computeMsSmooth = 0;
function step() {
  for (let i = 0; i < N; i++) {
    const p = nodes[i];
    let f = 0;
    if (i > 0) f += (nodes[i - 1].x - p.x) * k;
    if (i < N - 1) f += (nodes[i + 1].x - p.x) * k;
    p.v += f * dt;
  }
  for (let i = 0; i < N; i++) {
    nodes[i].x += nodes[i].v;
  }
}
function frame(now) {
  const t0 = performance.now();
  step();
  const compute = performance.now() - t0;
  computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;
  const W = canvas.width;
  const H = canvas.height;
  ctx.fillStyle = "#0d1117";
  ctx.fillRect(0, 0, W, H);
  ctx.strokeStyle = "rgba(88, 166, 255, 0.75)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  const stepX = W / N;
  for (let i = 0; i < N; i += 6) {
    const px = i * stepX;
    const py = H * 0.5 + nodes[i].x * 1.8;
    if (i === 0) ctx.moveTo(px, py);
    else ctx.lineTo(px, py);
  }
  ctx.stroke();
  ctx.fillStyle = "#58a6ff";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(`${N.toLocaleString()} springs \xB7 Node[] \xB7 TypeScript`, 8, 18);
  frames++;
  const dt2 = now - last;
  if (dt2 >= 500) {
    fpsSmooth = frames * 1e3 / dt2;
    frames = 0;
    last = now;
  }
  elFps.textContent = fpsSmooth.toFixed(0);
  elMs.textContent = computeMsSmooth.toFixed(2);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
