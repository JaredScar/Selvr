// game.ts
var N = 24e3;
var BX = 80;
var BY = 45;
var BUCK = BX * BY;
var buckets = [];
for (let b = 0; b < BUCK; b++) {
  buckets.push([]);
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
var seed = 123456789;
function work() {
  for (let b = 0; b < BUCK; b++) {
    buckets[b].length = 0;
  }
  let s = seed;
  for (let i = 0; i < N; i++) {
    s = Math.imul(s, 1664525) + 1013904223 >>> 0;
    const bid = s % BUCK;
    buckets[bid].push(i);
  }
  seed = seed + 2654435769 >>> 0;
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
  ctx.fillText(`${N.toLocaleString()} pts \u2192 ${BUCK} buckets \xB7 nested arrays \xB7 TypeScript`, 8, 20);
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
