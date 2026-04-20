// game.ts
var N = 8192;
var BATCH = 400;
var mem = new Float64Array(N);
function jsSum() {
  let s = 0;
  for (let i = 0; i < N; i++) {
    s += mem[i];
  }
  return s;
}
function work() {
  let acc = 0;
  for (let r = 0; r < BATCH; r++) {
    for (let i = 0; i < N; i++) {
      mem[i] = Math.random();
    }
    acc += jsSum();
  }
  return acc;
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
function frame(now) {
  const t0 = performance.now();
  const acc = work();
  const compute = performance.now() - t0;
  computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;
  const W = canvas.width;
  const H = canvas.height;
  ctx.fillStyle = "#0d1117";
  ctx.fillRect(0, 0, W, H);
  const bar = Math.min(1, Math.abs(acc) % 1);
  ctx.fillStyle = "rgba(88, 166, 255, 0.35)";
  ctx.fillRect(10, H - 28, (W - 20) * bar, 8);
  ctx.strokeStyle = "#58a6ff";
  ctx.strokeRect(10, H - 28, W - 20, 8);
  ctx.fillStyle = "#58a6ff";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(
    `${BATCH}\xD7 sum(${N}) f64 \xB7 JavaScript only \xB7 acc\u2248${acc.toExponential(2)}`,
    8,
    18
  );
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
