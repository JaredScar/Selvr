// game.ts
var ROUNDS = 14;
var OPS = 900;
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
  const q = [];
  for (let r = 0; r < ROUNDS; r++) {
    for (let i = 0; i < OPS; i++) {
      q.push(i + r * 1e5);
    }
    for (let i = 0; i < OPS; i++) {
      sink += q.shift();
    }
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
  ctx.fillText(`${ROUNDS}\xD7 (${OPS} push + ${OPS} shift) \xB7 Array.shift \xB7 TypeScript \xB7 ${sink.toFixed(0)}`, 8, 20);
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
