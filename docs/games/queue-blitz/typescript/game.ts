/**
 * Queue Blitz — TypeScript: Array.push + Array.shift (shift moves all remaining elements).
 */
const ROUNDS = 14;
const OPS = 900;

const canvas = document.getElementById("game-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");
const elFps = document.getElementById("fps")!;
const elMs = document.getElementById("compute-ms")!;

let last = performance.now();
let frames = 0;
let fpsSmooth = 60;
let computeMsSmooth = 0;
let sink = 0;

function work(): void {
  const q: number[] = [];
  for (let r = 0; r < ROUNDS; r++) {
    for (let i = 0; i < OPS; i++) {
      q.push(i + r * 100_000);
    }
    for (let i = 0; i < OPS; i++) {
      sink += q.shift()!;
    }
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
  ctx.fillText(`${ROUNDS}× (${OPS} push + ${OPS} shift) · Array.shift · TypeScript · ${sink.toFixed(0)}`, 8, 20);

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
