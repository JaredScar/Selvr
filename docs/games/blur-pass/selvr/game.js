/**
 * Blur Pass — Selvr-style: flat Uint8Array + swap buffers for 3×3 box blur.
 */
(() => {
  const W = 120;
  const H = 120;
  const L = W * H;
  let cur = new Uint8Array(L);
  let nxt = new Uint8Array(L);

  for (let i = 0; i < L; i++) {
    cur[i] = (Math.random() * 256) | 0;
  }

  const canvas = document.getElementById("game-canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("2d");
  const img = ctx.createImageData(W, H);
  const elFps = document.getElementById("fps");
  const elMs = document.getElementById("compute-ms");

  let last = performance.now();
  let frames = 0;
  let fpsSmooth = 60;
  let computeMsSmooth = 0;

  function blurPass() {
    const a = cur;
    const b = nxt;
    for (let y = 1; y < H - 1; y++) {
      const row = y * W;
      for (let x = 1; x < W - 1; x++) {
        const i = row + x;
        let s = 0;
        for (let dy = -1; dy <= 1; dy++) {
          const r = row + dy * W;
          for (let dx = -1; dx <= 1; dx++) {
            s += a[r + x + dx];
          }
        }
        b[i] = (s / 9) | 0;
      }
    }
    const t = cur;
    cur = nxt;
    nxt = t;
  }

  function frame(now) {
    const t0 = performance.now();
    blurPass();
    const compute = performance.now() - t0;
    computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;

    const d = img.data;
    for (let i = 0; i < L; i++) {
      const v = cur[i];
      const j = i * 4;
      d[j] = v;
      d[j + 1] = v;
      d[j + 2] = v;
      d[j + 3] = 255;
    }
    ctx.putImageData(img, 0, 0);

    ctx.fillStyle = "#00d4aa";
    ctx.font = "11px ui-monospace, monospace";
    ctx.fillText(`${W}×${H} blur · Uint8Array · Selvr-style`, 6, 14);

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
