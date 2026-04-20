/**
 * Fib Rush — Selvr-style: iterative Fibonacci in WebAssembly; JS draws stats.
 */
(async () => {
  const K = window.SelvrGamesWasm;
  const canvas = document.getElementById("game-canvas");
  const ctx = canvas.getContext("2d");
  const elFps = document.getElementById("fps");
  const elMs = document.getElementById("compute-ms");

  if (!K || !K.buildFibWasm || !K.tryInst) {
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

  if (!ctx) throw new Error("2d");

  const { buildFibWasm, tryInst } = K;
  const exp = await tryInst(buildFibWasm());

  const FIB_N = 35;
  const BATCH = 25000;

  let last = performance.now();
  let frames = 0;
  let fpsSmooth = 60;
  let computeMsSmooth = 0;

  if (!exp || !exp.fib) {
    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "#f85149";
    ctx.font = "13px ui-monospace, monospace";
    ctx.fillText("WebAssembly not available in this environment.", 12, 36);
    elFps.textContent = "—";
    elMs.textContent = "—";
    return;
  }

  const wasmFib = exp.fib.bind(exp);

  function work() {
    let s = 0;
    for (let r = 0; r < BATCH; r++) {
      s = (s + wasmFib(FIB_N)) | 0;
    }
    return s;
  }

  function frame(now) {
    const t0 = performance.now();
    const acc = work();
    const compute = performance.now() - t0;
    computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;

    const W = canvas.width;
    const H = canvas.height;
    ctx.fillStyle = "#0d1117";
    ctx.fillRect(0, 0, W, H);

    const bar = Math.min(1, (acc & 0xffff) / 65535);
    ctx.fillStyle = "rgba(74, 222, 128, 0.35)";
    ctx.fillRect(10, H - 28, (W - 20) * bar, 8);
    ctx.strokeStyle = "#4ade80";
    ctx.strokeRect(10, H - 28, W - 20, 8);

    ctx.fillStyle = "#4ade80";
    ctx.font = "12px ui-monospace, monospace";
    ctx.fillText(
      `${BATCH}× fib(${FIB_N}) · WASM i32 loop · sink=${acc}`,
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
