/**
 * Binary Hunt — Selvr-style: lower_bound on sorted Uint32Array (dense indices).
 */
(() => {
  const N = 65536;
  const arr = new Uint32Array(N);
  for (let i = 0; i < N; i++) {
    arr[i] = i * 2;
  }
  const QUERIES = 20000;
  const HI = N * 2 + 2048;

  function lowerBound(a, x) {
    let lo = 0;
    let hi = a.length;
    while (lo < hi) {
      const m = (lo + hi) >>> 1;
      if (a[m] < x) lo = m + 1;
      else hi = m;
    }
    return lo;
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
    for (let q = 0; q < QUERIES; q++) {
      const x = (Math.random() * HI) | 0;
      sink += lowerBound(arr, x);
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
    ctx.fillText(
      `${QUERIES} lower_bound / frame · N=${N} · Uint32Array · sink=${sink.toFixed(0)}`,
      8,
      22
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
})();
