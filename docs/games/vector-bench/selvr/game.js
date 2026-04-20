/**
 * Vector Bench — Selvr-style: packed Float64Array, no per-vector object headers.
 */
(() => {
  const N = 30_000;
  const buf = new Float64Array(N * 3);

  for (let i = 0; i < N; i++) {
    const ix = i * 3;
    buf[ix] = Math.random() - 0.5;
    buf[ix + 1] = Math.random() - 0.5;
    buf[ix + 2] = Math.random() - 0.5;
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
  let phase = 0;

  function work() {
    phase += 0.015;
    for (let i = 0; i < N; i++) {
      const ix = i * 3;
      let x = buf[ix] + Math.sin(phase + i * 0.07) * 0.002;
      let y = buf[ix + 1] + Math.cos(phase * 0.9 + i * 0.08) * 0.002;
      let z = buf[ix + 2] + Math.sin(phase * 0.5 + i * 0.06) * 0.002;
      const len = Math.sqrt(x * x + y * y + z * z) || 1;
      buf[ix] = x / len;
      buf[ix + 1] = y / len;
      buf[ix + 2] = z / len;
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
    ctx.strokeStyle = "rgba(0, 212, 170, 0.4)";
    ctx.lineWidth = 1;
    ctx.beginPath();
    const stride = Math.max(1, Math.floor(N / 200));
    for (let i = 0; i < N; i += stride) {
      const ix = i * 3;
      const px = ((buf[ix] + 1) * 0.5) * (W - 20) + 10;
      const py = ((buf[ix + 1] + 1) * 0.5) * (H - 40) + 30;
      if (i === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.stroke();

    ctx.fillStyle = "#00d4aa";
    ctx.font = "12px ui-monospace, monospace";
    ctx.fillText(`${N.toLocaleString()}× normalize · packed f64 · Selvr-style`, 8, 18);

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
