/**
 * Dot Wave — Selvr-style: float dot product runs in WebAssembly; JS draws canvas.
 */
(async () => {
  const K = window.SelvrGamesWasm;
  const canvas = document.getElementById("game-canvas");
  const ctx = canvas.getContext("2d");
  const elFps = document.getElementById("fps");
  const elMs = document.getElementById("compute-ms");

  if (!K || !K.buildDotWasm || !K.tryInst) {
    if (ctx) {
      ctx.fillStyle = "#0d1117";
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = "#f85149";
      ctx.font = "12px ui-monospace, monospace";
      ctx.fillText("Could not load ../../_shared/wasm-kernels.js — check the script path.", 12, 28);
    }
    if (elFps) elFps.textContent = "—";
    if (elMs) elMs.textContent = "—";
    return;
  }

  const { buildDotWasm, tryInst } = K;
  const exp = await tryInst(buildDotWasm());

  if (!ctx) throw new Error("2d");

  const VLEN = 512;
  const BATCH = 4000;

  let last = performance.now();
  let frames = 0;
  let fpsSmooth = 60;
  let computeMsSmooth = 0;
  let phase = 0;

  if (!exp || !exp.dot || !exp.memory) {
    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "#f85149";
    ctx.font = "13px ui-monospace, monospace";
    ctx.fillText("WebAssembly not available in this environment.", 12, 36);
    elFps.textContent = "—";
    elMs.textContent = "—";
    return;
  }

  const mem = new Float64Array(exp.memory.buffer);
  const ptrA = 0;
  const ptrB = VLEN * 8;
  const dot = exp.dot.bind(exp);

  function refill() {
    for (let i = 0; i < VLEN; i++) {
      mem[i] = Math.random();
      mem[VLEN + i] = Math.random();
    }
  }
  refill();

  function work() {
    let acc = 0;
    for (let r = 0; r < BATCH; r++) {
      acc += dot(ptrA, ptrB, VLEN);
    }
    return acc;
  }

  function frame(now) {
    phase += 0.02;
    refill();

    const t0 = performance.now();
    const acc = work();
    const compute = performance.now() - t0;
    computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;

    const W = canvas.width;
    const H = canvas.height;
    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, W, H);

    const bar = Math.min(1, (Math.abs(acc) % 1000) / 1000);
    ctx.fillStyle = "rgba(0, 212, 170, 0.35)";
    ctx.fillRect(10, H - 28, (W - 20) * bar, 8);
    ctx.strokeStyle = "#00d4aa";
    ctx.strokeRect(10, H - 28, W - 20, 8);

    ctx.fillStyle = "#00d4aa";
    ctx.font = "12px ui-monospace, monospace";
    ctx.fillText(
      `${BATCH}× dot(${VLEN}) · WASM compute + JS canvas · acc≈${acc.toExponential(2)}`,
      8,
      18
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
