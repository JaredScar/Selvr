/**
 * Alloc Strike — TypeScript: allocate a new Float64Array(N) every frame (GC pressure).
 */
const N = 65536;
const buf = new Float64Array(N);
for (let i = 0; i < N; i++) buf[i] = Math.random();

const canvas = document.getElementById("game-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");
const elFps = document.getElementById("fps")!;
const elMs = document.getElementById("compute-ms")!;

let last = performance.now();
let frames = 0;
let fpsSmooth = 60;
let computeMsSmooth = 0;
let phase = 0;

function work(): void {
  phase += 0.01;
  const out = new Float64Array(N);
  for (let i = 0; i < N; i++) {
    out[i] = Math.sin(phase + i * 0.0001) * 0.5 + buf[i] * 0.5;
  }
}

function frame(now: number): void {
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
  ctx.fillText(`${N.toLocaleString()} samples · new Float64Array per frame · TypeScript`, 8, 20);

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
