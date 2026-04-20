// game.ts
var N = 65536;
var arr = [];
for (let i = 0; i < N; i++) {
  arr.push(i * 2);
}
var QUERIES = 2e4;
var HI = N * 2 + 2048;
function lowerBound(a, x) {
  let lo = 0;
  let hi = a.length;
  while (lo < hi) {
    const m = lo + hi >>> 1;
    if (a[m] < x) lo = m + 1;
    else hi = m;
  }
  return lo;
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
var sink = 0;
function work() {
  for (let q = 0; q < QUERIES; q++) {
    const x = Math.random() * HI | 0;
    sink += lowerBound(arr, x);
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
  ctx.fillText(
    `${QUERIES} lower_bound / frame \xB7 N=${N} \xB7 number[] \xB7 sink=${sink.toFixed(0)}`,
    8,
    22
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
