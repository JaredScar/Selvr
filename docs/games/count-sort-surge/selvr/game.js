/**
 * Count Sort Surge — Selvr-style: Uint32Array values + count / output buffers.
 */
(() => {
  const N = 100_000;
  const K = 2048;
  const vals = new Uint32Array(N);
  const cnt = new Uint32Array(K);
  const out = new Uint32Array(N);

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
    let s = 2463534242;
    for (let i = 0; i < N; i++) {
      s ^= s << 13;
      s >>>= 0;
      s ^= s >>> 17;
      s ^= s << 5;
      s >>>= 0;
      vals[i] = (s >>> 0) % K;
    }

    cnt.fill(0);
    for (let i = 0; i < N; i++) cnt[vals[i]]++;

    let acc = 0;
    for (let i = 0; i < K; i++) {
      const t = cnt[i];
      cnt[i] = acc;
      acc += t;
    }

    for (let i = 0; i < N; i++) {
      const v = vals[i];
      out[cnt[v]++] = v;
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
    ctx.fillText(`${N.toLocaleString()} keys · counting sort · Uint32Array · Selvr-style`, 8, 20);

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
