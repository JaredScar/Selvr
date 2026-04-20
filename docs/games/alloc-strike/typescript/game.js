// game.ts
var N = 65536;
var buf = new Float64Array(N);
for (let i = 0; i < N; i++) buf[i] = Math.random();
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
  phase += 0.01;
  const out = new Float64Array(N);
  for (let i = 0; i < N; i++) {
    out[i] = Math.sin(phase + i * 1e-4) * 0.5 + buf[i] * 0.5;
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
  ctx.fillStyle = "#58a6ff";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(`${N.toLocaleString()} samples \xB7 new Float64Array per frame \xB7 TypeScript`, 8, 20);
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
