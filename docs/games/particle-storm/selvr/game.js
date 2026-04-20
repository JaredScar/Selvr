/**
 * Particle Storm — Selvr-style hot path (struct-of-arrays, zero heap churn per frame).
 * Mirrors what the Selvr compiler emits: packed f64 buffers, tight numeric loops.
 */
(() => {
  const W = 560;
  const H = 420;
  const N = 48_000;

  const xs = new Float64Array(N);
  const ys = new Float64Array(N);
  const vxs = new Float64Array(N);
  const vys = new Float64Array(N);

  for (let i = 0; i < N; i++) {
    xs[i] = Math.random() * W;
    ys[i] = Math.random() * H;
    vxs[i] = (Math.random() - 0.5) * 3.5;
    vys[i] = (Math.random() - 0.5) * 3.5;
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

  /** Draw a sparse sample so the GPU/Canvas isn't the bottleneck — physics stays full-N. */
  const STRIDE = 24;

  function frame(now) {
    const t0 = performance.now();

    for (let i = 0; i < N; i++) {
      xs[i] += vxs[i];
      ys[i] += vys[i];
      if (xs[i] < 0) {
        xs[i] = 0;
        vxs[i] *= -1;
      } else if (xs[i] > W) {
        xs[i] = W;
        vxs[i] *= -1;
      }
      if (ys[i] < 0) {
        ys[i] = 0;
        vys[i] *= -1;
      } else if (ys[i] > H) {
        ys[i] = H;
        vys[i] *= -1;
      }
    }

    const compute = performance.now() - t0;
    computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;

    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, W, H);
    ctx.fillStyle = "rgba(0, 212, 170, 0.55)";
    for (let i = 0; i < N; i += STRIDE) {
      ctx.fillRect(xs[i], ys[i], 1.5, 1.5);
    }

    ctx.fillStyle = "#8b949e";
    ctx.font = "12px ui-monospace, monospace";
    ctx.fillText(`${N.toLocaleString()} particles · SoA buffers · Selvr-style`, 8, 18);

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

  canvas.width = W;
  canvas.height = H;
  requestAnimationFrame(frame);
})();
