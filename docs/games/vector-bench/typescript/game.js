// game.ts
var N = 3e4;
var vecs = [];
for (let i = 0; i < N; i++) {
  vecs.push({
    x: Math.random() - 0.5,
    y: Math.random() - 0.5,
    z: Math.random() - 0.5
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
var phase = 0;
function work() {
  phase += 0.015;
  for (let i = 0; i < N; i++) {
    const v = vecs[i];
    let x = v.x + Math.sin(phase + i * 0.07) * 2e-3;
    let y = v.y + Math.cos(phase * 0.9 + i * 0.08) * 2e-3;
    let z = v.z + Math.sin(phase * 0.5 + i * 0.06) * 2e-3;
    const len = Math.sqrt(x * x + y * y + z * z) || 1;
    v.x = x / len;
    v.y = y / len;
    v.z = z / len;
  }
}
function frame(now) {
  const t0 = performance.now();
  work();
  const compute = performance.now() - t0;
  computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;
  const W = canvas.width;
  const H = canvas.height;
  ctx.fillStyle = "#0d1117";
  ctx.fillRect(0, 0, W, H);
  ctx.strokeStyle = "rgba(88, 166, 255, 0.45)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  const stride = Math.max(1, Math.floor(N / 200));
  for (let i = 0; i < N; i += stride) {
    const v = vecs[i];
    const px = (v.x + 1) * 0.5 * (W - 20) + 10;
    const py = (v.y + 1) * 0.5 * (H - 40) + 30;
    if (i === 0) ctx.moveTo(px, py);
    else ctx.lineTo(px, py);
  }
  ctx.stroke();
  ctx.fillStyle = "#58a6ff";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(`${N.toLocaleString()}\xD7 normalize \xB7 Vec3[] \xB7 TypeScript`, 8, 18);
  frames++;
  const dt = now - last;
  if (dt >= 500) {
    fpsSmooth = frames * 1e3 / dt;
    frames = 0;
    last = now;
  }
  elFps.textContent = fpsSmooth.toFixed(0);
  elMs.textContent = computeMsSmooth.toFixed(2);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
