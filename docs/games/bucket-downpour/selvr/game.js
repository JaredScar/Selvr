/**
 * Bucket Downpour — Selvr-style: count buckets then scatter indices into flat Int32Array.
 */
(() => {
  const N = 24_000;
  const BX = 80;
  const BY = 45;
  const BUCK = BX * BY;

  const counts = new Int32Array(BUCK);
  const flat = new Int32Array(N);

  const canvas = document.getElementById("game-canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("2d");
  const elFps = document.getElementById("fps");
  const elMs = document.getElementById("compute-ms");

  let last = performance.now();
  let frames = 0;
  let fpsSmooth = 60;
  let computeMsSmooth = 0;
  let seed = 123456789;

  function work() {
    counts.fill(0);
    let s = seed;
    for (let i = 0; i < N; i++) {
      s = (Math.imul(s, 1664525) + 1013904223) >>> 0;
      const bid = s % BUCK;
      counts[bid]++;
    }

    let acc = 0;
    for (let b = 0; b < BUCK; b++) {
      const t = counts[b];
      counts[b] = acc;
      acc += t;
    }

    s = seed;
    for (let i = 0; i < N; i++) {
      s = (Math.imul(s, 1664525) + 1013904223) >>> 0;
      const bid = s % BUCK;
      flat[counts[bid]++] = i;
    }

    seed = (seed + 0x9e3779b9) >>> 0;
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
    ctx.fillText(`${N.toLocaleString()} pts → ${BUCK} buckets · flat scatter · Selvr-style`, 8, 20);

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
