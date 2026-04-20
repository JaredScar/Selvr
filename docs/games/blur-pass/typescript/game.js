// game.ts
var W = 120;
var H = 120;
function makeGrid() {
  const g = [];
  for (let y = 0; y < H; y++) {
    const row = [];
    for (let x = 0; x < W; x++) {
      row.push(Math.random() * 256 | 0);
    }
    g.push(row);
  }
  return g;
}
function makeEmptyGrid() {
  const g = [];
  for (let y = 0; y < H; y++) {
    g.push(Array(W).fill(0));
  }
  return g;
}
var cur = makeGrid();
var nxt = makeEmptyGrid();
var canvas = document.getElementById("game-canvas");
var ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");
var elFps = document.getElementById("fps");
var elMs = document.getElementById("compute-ms");
var img = ctx.createImageData(W, H);
var last = performance.now();
var frames = 0;
var fpsSmooth = 60;
var computeMsSmooth = 0;
function blurPass() {
  const a = cur;
  const b = nxt;
  for (let y = 1; y < H - 1; y++) {
    for (let x = 1; x < W - 1; x++) {
      let s = 0;
      for (let dy = -1; dy <= 1; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          s += a[y + dy][x + dx];
        }
      }
      b[y][x] = s / 9 | 0;
    }
  }
  const t = cur;
  cur = nxt;
  nxt = t;
}
function frame(now) {
  const t0 = performance.now();
  blurPass();
  const compute = performance.now() - t0;
  computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;
  const d = img.data;
  for (let y = 0; y < H; y++) {
    const row = cur[y];
    for (let x = 0; x < W; x++) {
      const v = row[x];
      const i = (y * W + x) * 4;
      d[i] = v;
      d[i + 1] = v;
      d[i + 2] = v;
      d[i + 3] = 255;
    }
  }
  ctx.putImageData(img, 0, 0);
  ctx.fillStyle = "#58a6ff";
  ctx.font = "11px ui-monospace, monospace";
  ctx.fillText(`${W}\xD7${H} blur \xB7 number[][] \xB7 TypeScript`, 6, 14);
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
