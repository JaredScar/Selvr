/**
 * Path BFS — TypeScript: boolean[][] visited + queue with Array.shift() (O(n) per dequeue).
 */
const W = 56;
const H = 56;

const visited: boolean[][] = [];
for (let y = 0; y < H; y++) {
  visited.push(Array<boolean>(W).fill(false));
}

const canvas = document.getElementById("game-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");
const elFps = document.getElementById("fps")!;
const elMs = document.getElementById("compute-ms")!;

let last = performance.now();
let frames = 0;
let fpsSmooth = 60;
let computeMsSmooth = 0;

function work(): void {
  for (let y = 0; y < H; y++) {
    const row = visited[y];
    for (let x = 0; x < W; x++) row[x] = false;
  }
  const q: number[] = [];
  const sy = (H / 2) | 0;
  const sx = (W / 2) | 0;
  q.push(sy * W + sx);
  visited[sy][sx] = true;

  while (q.length > 0) {
    const cur = q.shift()!;
    const x = cur % W;
    const y = (cur / W) | 0;
    if (x > 0) {
      const n = cur - 1;
      if (!visited[y][x - 1]) {
        visited[y][x - 1] = true;
        q.push(n);
      }
    }
    if (x + 1 < W) {
      const n = cur + 1;
      if (!visited[y][x + 1]) {
        visited[y][x + 1] = true;
        q.push(n);
      }
    }
    if (y > 0) {
      const n = cur - W;
      if (!visited[y - 1][x]) {
        visited[y - 1][x] = true;
        q.push(n);
      }
    }
    if (y + 1 < H) {
      const n = cur + W;
      if (!visited[y + 1][x]) {
        visited[y + 1][x] = true;
        q.push(n);
      }
    }
  }
}

function frame(now: number): void {
  const t0 = performance.now();
  work();
  const compute = performance.now() - t0;
  computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;

  const CW = canvas.width;
  const CH = canvas.height;
  const scaleX = CW / W;
  const scaleY = CH / H;
  ctx.fillStyle = "#0d1117";
  ctx.fillRect(0, 0, CW, CH);
  ctx.fillStyle = "rgba(88, 166, 255, 0.85)";
  for (let y = 0; y < H; y++) {
    const row = visited[y];
    for (let x = 0; x < W; x++) {
      if (row[x]) {
        ctx.fillRect(x * scaleX, y * scaleY, scaleX, scaleY);
      }
    }
  }
  ctx.fillStyle = "#8b949e";
  ctx.font = "11px ui-monospace, monospace";
  ctx.fillText(`${W}×${H} BFS · shift() queue · TypeScript`, 6, 14);

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
