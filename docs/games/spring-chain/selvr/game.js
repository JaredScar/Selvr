/**
 * Spring Chain — Selvr-style: parallel Float64Array for position and velocity.
 */
(() => {
  const N = 3800;
  const x = new Float64Array(N);
  const v = new Float64Array(N);
  const k = 0.18;
  const dt = 0.12;

  for (let i = 0; i < N; i++) {
    x[i] = Math.sin(i * 0.018) * 24 + (Math.random() - 0.5) * 2;
    v[i] = 0;
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

  function step() {
    for (let i = 0; i < N; i++) {
      let f = 0;
      if (i > 0) f += (x[i - 1] - x[i]) * k;
      if (i < N - 1) f += (x[i + 1] - x[i]) * k;
      v[i] += f * dt;
    }
    for (let i = 0; i < N; i++) {
      x[i] += v[i];
    }
  }

  function frame(now) {
    const t0 = performance.now();
    step();
    const compute = performance.now() - t0;
    computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;

    const W = canvas.width;
    const H = canvas.height;
    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, W, H);
    ctx.strokeStyle = "rgba(0, 212, 170, 0.75)";
    ctx.lineWidth = 1;
    ctx.beginPath();
    const stepX = W / N;
    for (let i = 0; i < N; i += 6) {
      const px = i * stepX;
      const py = H * 0.5 + x[i] * 1.8;
      if (i === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.stroke();

    ctx.fillStyle = "#00d4aa";
    ctx.font = "12px ui-monospace, monospace";
    ctx.fillText(`${N.toLocaleString()} springs · SoA f64 · Selvr-style`, 8, 18);

    frames++;
    const dt2 = now - last;
    if (dt2 >= 500) {
      fpsSmooth = (frames * 1000) / dt2;
      frames = 0;
      last = now;
    }
    elFps.textContent = fpsSmooth.toFixed(0);
    elMs.textContent = computeMsSmooth.toFixed(2);
    requestAnimationFrame(frame);
  }

  requestAnimationFrame(frame);
})();
