// game.ts
var FIB_N = 35;
var BATCH = 25e3;
function jsFib(n) {
  let a = 0;
  let b = 1;
  for (let i = 0; i < n; i++) {
    const t = a + b;
    a = b;
    b = t;
  }
  return b;
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
function work() {
  let s = 0;
  for (let r = 0; r < BATCH; r++) {
    s = s + jsFib(FIB_N) | 0;
  }
  return s;
}
function frame(now) {
  const t0 = performance.now();
  const acc = work();
  const compute = performance.now() - t0;
  computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;
  const W = canvas.width;
  const H = canvas.height;
  ctx.fillStyle = "#0d1117";
  ctx.fillRect(0, 0, W, H);
  const bar = Math.min(1, (acc & 65535) / 65535);
  ctx.fillStyle = "rgba(88, 166, 255, 0.35)";
  ctx.fillRect(10, H - 28, (W - 20) * bar, 8);
  ctx.strokeStyle = "#58a6ff";
  ctx.strokeRect(10, H - 28, W - 20, 8);
  ctx.fillStyle = "#58a6ff";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(`${BATCH}\xD7 fib(${FIB_N}) \xB7 JavaScript only \xB7 sink=${acc}`, 8, 18);
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
