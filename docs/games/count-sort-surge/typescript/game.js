// game.ts
var N = 1e5;
var K = 2048;
var vals = new Array(N);
var cnt = new Array(K);
var out = new Array(N);
var canvas = document.getElementById("game-canvas");
var ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");
var elFps = document.getElementById("fps");
var elMs = document.getElementById("compute-ms");
var last = performance.now();
var frames = 0;
var fpsSmooth = 60;
var computeMsSmooth = 0;
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
  for (let i = 0; i < K; i++) cnt[i] = 0;
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
  ctx.fillStyle = "#58a6ff";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(`${N.toLocaleString()} keys \xB7 counting sort \xB7 number[] \xB7 TypeScript`, 8, 20);
  frames++;
  const dt = now - last;
  if (dt >= 500) {
    fpsSmooth = frames * 1e3 / dt;
    frames = 0;
    last = now;
  }
  elFps.textContent = fpsSmooth.toFixed(0);
  elMs.textContent = computeMsSmooth.toFixed(2);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
