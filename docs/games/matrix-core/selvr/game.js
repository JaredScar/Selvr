/**
 * Matrix Core — Selvr-style: row-major Float64Array for A, B, C (dense matmul).
 */
(() => {
  const S = 32;
  const SS = S * S;
  const IT = 56;

  const A = new Float64Array(SS);
  const B = new Float64Array(SS);
  const C = new Float64Array(SS);

  for (let i = 0; i < SS; i++) {
    A[i] = Math.random();
    B[i] = Math.random();
  }

  function mmul() {
    for (let i = 0; i < S; i++) {
      const row = i * S;
      for (let j = 0; j < S; j++) {
        let sum = 0;
        for (let k = 0; k < S; k++) {
          sum += A[row + k] * B[k * S + j];
        }
        C[row + j] = sum;
      }
    }
  }

  const canvas = document.getElementById("game-canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("2d");
  const elFps = document.getElementById("fps");
  const elMs = document.getElementById("compute-ms");

  let last = performance.now();
  let frames = 0;
  let fpsSmooth = 60;
  let computeMsSmooth = 0;
  let sink = 0;

  function work() {
    for (let t = 0; t < IT; t++) {
      mmul();
      sink += C[0];
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
    ctx.fillStyle = "#00d4aa";
    ctx.font = "12px ui-monospace, monospace";
    ctx.fillText(`${S}×${S} · ${IT}× matmul · row-major f64 · Selvr-style · ${sink.toFixed(0)}`, 8, 20);

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
})();
