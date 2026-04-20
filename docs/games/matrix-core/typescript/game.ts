/**
 * Matrix Core — TypeScript: number[][] row-of-rows for A, B, C.
 */
const S = 32;
const IT = 56;

function mat(): number[][] {
  const m: number[][] = [];
  for (let i = 0; i < S; i++) {
    const row: number[] = [];
    for (let j = 0; j < S; j++) row.push(Math.random());
    m.push(row);
  }
  return m;
}

const A = mat();
const B = mat();
const C = mat();

function mmul(): void {
  for (let i = 0; i < S; i++) {
    for (let j = 0; j < S; j++) {
      let sum = 0;
      for (let k = 0; k < S; k++) {
        sum += A[i][k] * B[k][j];
      }
      C[i][j] = sum;
    }
  }
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
let sink = 0;

function work(): void {
  for (let t = 0; t < IT; t++) {
    mmul();
    sink += C[0][0];
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
  ctx.fillText(`${S}×${S} · ${IT}× matmul · number[][] · TypeScript · ${sink.toFixed(0)}`, 8, 20);

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
