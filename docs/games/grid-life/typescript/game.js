// game.ts
var W = 180;
var H = 200;
function makeGrid() {
  const g = [];
  for (let y = 0; y < H; y++) {
    const row = [];
    for (let x = 0; x < W; x++) {
      row.push(Math.random() < 0.18);
    }
    g.push(row);
  }
  return g;
}
var cur = makeGrid();
var next = [];
for (let y = 0; y < H; y++) {
  next.push(Array(W).fill(false));
}
var canvas = document.getElementById("game-canvas");
var ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");
var elFps = document.getElementById("fps");
var elMs = document.getElementById("compute-ms");
var SCALE = 2;
var CW = W * SCALE;
var CH = H * SCALE;
var last = performance.now();
var frames = 0;
var fpsSmooth = 60;
var computeMsSmooth = 0;
function step() {
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      let cnt = 0;
      for (let dy = -1; dy <= 1; dy++) {
        const ny = y + dy;
        if (ny < 0 || ny >= H) continue;
        const row = cur[ny];
        for (let dx = -1; dx <= 1; dx++) {
          if (dx === 0 && dy === 0) continue;
          const nx = x + dx;
          if (nx < 0 || nx >= W) continue;
          if (row[nx]) cnt++;
        }
      }
      const alive = cur[y][x];
      if (alive) {
        next[y][x] = cnt === 2 || cnt === 3;
      } else {
        next[y][x] = cnt === 3;
      }
    }
  }
  for (let y = 0; y < H; y++) {
    const cr = cur[y];
    const nr = next[y];
    for (let x = 0; x < W; x++) {
      cr[x] = nr[x];
    }
  }
}
function frame(now) {
  const t0 = performance.now();
  step();
  const compute = performance.now() - t0;
  computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;
  ctx.fillStyle = "#0d1117";
  ctx.fillRect(0, 0, CW, CH);
  ctx.fillStyle = "rgba(88, 166, 255, 0.85)";
  for (let y = 0; y < H; y++) {
    const row = cur[y];
    for (let x = 0; x < W; x++) {
      if (row[x]) {
        ctx.fillRect(x * SCALE, y * SCALE, SCALE, SCALE);
      }
    }
  }
  ctx.fillStyle = "#8b949e";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(`${W}\xD7${H} Life \xB7 boolean[][] \xB7 TypeScript`, 8, 18);
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
