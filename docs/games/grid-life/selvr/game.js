/**
 * Grid Life — Selvr-style: flat Uint8Array buffers, index math, swap without GC.
 */
(() => {
  const W = 180;
  const H = 200;
  const LEN = W * H;

  const cur = new Uint8Array(LEN);
  const next = new Uint8Array(LEN);

  for (let i = 0; i < LEN; i++) {
    cur[i] = Math.random() < 0.18 ? 1 : 0;
  }

  const canvas = document.getElementById("game-canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("2d");

  const elFps = document.getElementById("fps");
  const elMs = document.getElementById("compute-ms");

  const SCALE = 2;
  const CW = W * SCALE;
  const CH = H * SCALE;

  let last = performance.now();
  let frames = 0;
  let fpsSmooth = 60;
  let computeMsSmooth = 0;

  function step() {
    const c = cur;
    const n = next;
    for (let y = 0; y < H; y++) {
      for (let x = 0; x < W; x++) {
        const i = y * W + x;
        let cnt = 0;
        for (let dy = -1; dy <= 1; dy++) {
          const ny = y + dy;
          if (ny < 0 || ny >= H) continue;
          const base = ny * W;
          for (let dx = -1; dx <= 1; dx++) {
            if (dx === 0 && dy === 0) continue;
            const nx = x + dx;
            if (nx < 0 || nx >= W) continue;
            cnt += c[base + nx];
          }
        }
        const alive = c[i];
        let v = 0;
        if (alive) {
          if (cnt === 2 || cnt === 3) v = 1;
        } else if (cnt === 3) {
          v = 1;
        }
        n[i] = v;
      }
    }
    cur.set(n);
  }

  function frame(now) {
    const t0 = performance.now();
    step();
    const compute = performance.now() - t0;
    computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;

    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, CW, CH);
    ctx.fillStyle = "rgba(0, 212, 170, 0.85)";
    for (let y = 0; y < H; y++) {
      const row = y * W;
      for (let x = 0; x < W; x++) {
        if (cur[row + x]) {
          ctx.fillRect(x * SCALE, y * SCALE, SCALE, SCALE);
        }
      }
    }

    ctx.fillStyle = "#8b949e";
    ctx.font = "12px ui-monospace, monospace";
    ctx.fillText(`${W}×${H} Life · flat Uint8Array · Selvr-style`, 8, 18);

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
