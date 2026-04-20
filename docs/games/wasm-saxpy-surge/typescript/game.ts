/**
 * Saxpy Surge — JavaScript-only y += a*x.
 */
const N = 4096;
const BATCH = 100;
const ALPHA = 0.75;

const x = new Float64Array(N);
const y = new Float64Array(N);

function work(): number {
  let sink = 0;
  for (let r = 0; r < BATCH; r++) {
    for (let i = 0; i < N; i++) {
      x[i] = Math.random();
      y[i] = Math.random();
    }
    for (let i = 0; i < N; i++) {
      y[i] += ALPHA * x[i];
    }
    sink += y[0];
  }
  return sink;
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

function frame(now: number): void {
  const t0 = performance.now();
  const acc = work();
  const compute = performance.now() - t0;
  computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;

  const W = canvas.width;
  const H = canvas.height;
  ctx.fillStyle = "#0d1117";
  ctx.fillRect(0, 0, W, H);

  const bar = Math.min(1, Math.abs(acc % 1024) / 1024);
  ctx.fillStyle = "rgba(88, 166, 255, 0.35)";
  ctx.fillRect(10, H - 28, (W - 20) * bar, 8);
  ctx.strokeStyle = "#58a6ff";
  ctx.strokeRect(10, H - 28, W - 20, 8);

  ctx.fillStyle = "#58a6ff";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(
    `${BATCH}× saxpy(${N}) · JavaScript only · acc≈${acc.toExponential(2)}`,
    8,
    18
  );

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
