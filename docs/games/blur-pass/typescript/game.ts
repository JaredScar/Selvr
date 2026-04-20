/**
 * Blur Pass — TypeScript: nested number[][] for grid + 3×3 box blur.
 */
const W = 120;
const H = 120;

function makeGrid(): number[][] {
  const g: number[][] = [];
  for (let y = 0; y < H; y++) {
    const row: number[] = [];
    for (let x = 0; x < W; x++) {
      row.push((Math.random() * 256) | 0);
    }
    g.push(row);
  }
  return g;
}

function makeEmptyGrid(): number[][] {
  const g: number[][] = [];
  for (let y = 0; y < H; y++) {
    g.push(Array<number>(W).fill(0));
  }
  return g;
}

let cur = makeGrid();
let nxt = makeEmptyGrid();

const canvas = document.getElementById("game-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");

const elFps = document.getElementById("fps")!;
const elMs = document.getElementById("compute-ms")!;

const img = ctx.createImageData(W, H);

let last = performance.now();
let frames = 0;
let fpsSmooth = 60;
let computeMsSmooth = 0;

function blurPass(): void {
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
      b[y][x] = (s / 9) | 0;
    }
  }
  const t = cur;
  cur = nxt;
  nxt = t;
}

function frame(now: number): void {
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
  ctx.fillText(`${W}×${H} blur · number[][] · TypeScript`, 6, 14);

  frames++;
  const dt = now - last;
  if (dt >= 500) {
    fpsSmooth = (frames * 1000) / dt;
    frames = 0;
    last = now;
  }
  elFps.textContent = fpsSmooth.toFixed(0);
  elMs.textContent = computeMsSmooth.toFixed(2);
  requestAnimationFrame(frame);
}

requestAnimationFrame(frame);
