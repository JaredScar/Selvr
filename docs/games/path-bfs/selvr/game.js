/**
 * Path BFS — Selvr-style: Uint8Array visited + Int32Array ring queue (head/tail, no shift).
 */
(() => {
  const W = 56;
  const H = 56;
  const L = W * H;
  const visited = new Uint8Array(L);
  const q = new Int32Array(L + 4);

  const canvas = document.getElementById("game-canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("2d");
  const elFps = document.getElementById("fps");
  const elMs = document.getElementById("compute-ms");

  let last = performance.now();
  let frames = 0;
  let fpsSmooth = 60;
  let computeMsSmooth = 0;

  function work() {
    visited.fill(0);
    let head = 0;
    let tail = 0;
    const sy = (H / 2) | 0;
    const sx = (W / 2) | 0;
    const start = sy * W + sx;
    q[tail++] = start;
    visited[start] = 1;

    while (head < tail) {
      const cur = q[head++];
      const x = cur % W;
      const y = (cur / W) | 0;
      if (x > 0) {
        const n = cur - 1;
        if (!visited[n]) {
          visited[n] = 1;
          q[tail++] = n;
        }
      }
      if (x + 1 < W) {
        const n = cur + 1;
        if (!visited[n]) {
          visited[n] = 1;
          q[tail++] = n;
        }
      }
      if (y > 0) {
        const n = cur - W;
        if (!visited[n]) {
          visited[n] = 1;
          q[tail++] = n;
        }
      }
      if (y + 1 < H) {
        const n = cur + W;
        if (!visited[n]) {
          visited[n] = 1;
          q[tail++] = n;
        }
      }
    }
  }

  function frame(now) {
    const t0 = performance.now();
    work();
    const compute = performance.now() - t0;
    computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;

    const CW = canvas.width;
    const CH = canvas.height;
    const scaleX = CW / W;
    const scaleY = CH / H;
    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, CW, CH);
    ctx.fillStyle = "rgba(0, 212, 170, 0.85)";
    for (let i = 0; i < L; i++) {
      if (visited[i]) {
        const x = i % W;
        const y = (i / W) | 0;
        ctx.fillRect(x * scaleX, y * scaleY, scaleX, scaleY);
      }
    }
    ctx.fillStyle = "#8b949e";
    ctx.font = "11px ui-monospace, monospace";
    ctx.fillText(`${W}×${H} BFS · ring queue · Selvr-style`, 6, 14);

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
