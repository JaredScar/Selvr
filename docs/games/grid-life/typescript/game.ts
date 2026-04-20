/**
 * Grid Life — TypeScript: nested boolean[][] (typical OO / grid-of-rows pattern).
 */
const W = 180;
const H = 200;

function makeGrid(): boolean[][] {
  const g: boolean[][] = [];
  for (let y = 0; y < H; y++) {
    const row: boolean[] = [];
    for (let x = 0; x < W; x++) {
      row.push(Math.random() < 0.18);
    }
    g.push(row);
  }
  return g;
}

const cur = makeGrid();
const next: boolean[][] = [];
for (let y = 0; y < H; y++) {
  next.push(Array<boolean>(W).fill(false));
}

const canvas = document.getElementById("game-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");

const elFps = document.getElementById("fps")!;
const elMs = document.getElementById("compute-ms")!;

const SCALE = 2;
const CW = W * SCALE;
const CH = H * SCALE;

let last = performance.now();
let frames = 0;
let fpsSmooth = 60;
let computeMsSmooth = 0;

function step(): void {
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

function frame(now: number): void {
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
  ctx.fillText(`${W}×${H} Life · boolean[][] · TypeScript`, 8, 18);

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
