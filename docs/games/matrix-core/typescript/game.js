// game.ts
var S = 32;
var IT = 56;
function mat() {
  const m = [];
  for (let i = 0; i < S; i++) {
    const row = [];
    for (let j = 0; j < S; j++) row.push(Math.random());
    m.push(row);
  }
  return m;
}
var A = mat();
var B = mat();
var C = mat();
function mmul() {
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
  for (let t = 0; t < IT; t++) {
    mmul();
    sink += C[0][0];
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
  ctx.fillText(`${S}\xD7${S} \xB7 ${IT}\xD7 matmul \xB7 number[][] \xB7 TypeScript \xB7 ${sink.toFixed(0)}`, 8, 20);
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
