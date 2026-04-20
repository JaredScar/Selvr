// game.ts
var W = 560;
var H = 420;
var N = 48e3;
var Particle = class {
  x = 0;
  y = 0;
  vx = 0;
  vy = 0;
};
var particles = [];
for (let i = 0; i < N; i++) {
  const p = new Particle();
  p.x = Math.random() * W;
  p.y = Math.random() * H;
  p.vx = (Math.random() - 0.5) * 3.5;
  p.vy = (Math.random() - 0.5) * 3.5;
  particles.push(p);
}
var canvas = document.getElementById("game-canvas");
var ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");
var elFps = document.getElementById("fps");
var elMs = document.getElementById("compute-ms");
var last = performance.now();
var frames = 0;
var fpsSmooth = 60;
var computeMsSmooth = 0;
var STRIDE = 24;
function frame(now) {
  const t0 = performance.now();
  for (let i = 0; i < particles.length; i++) {
    const p = particles[i];
    p.x += p.vx;
    p.y += p.vy;
    if (p.x < 0) {
      p.x = 0;
      p.vx *= -1;
    } else if (p.x > W) {
      p.x = W;
      p.vx *= -1;
    }
    if (p.y < 0) {
      p.y = 0;
      p.vy *= -1;
    } else if (p.y > H) {
      p.y = H;
      p.vy *= -1;
    }
  }
  const compute = performance.now() - t0;
  computeMsSmooth = computeMsSmooth * 0.92 + compute * 0.08;
  ctx.fillStyle = "#0d1117";
  ctx.fillRect(0, 0, W, H);
  ctx.fillStyle = "rgba(88, 166, 255, 0.55)";
  for (let i = 0; i < particles.length; i += STRIDE) {
    const p = particles[i];
    ctx.fillRect(p.x, p.y, 1.5, 1.5);
  }
  ctx.fillStyle = "#8b949e";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(`${N.toLocaleString()} particles \xB7 heap objects \xB7 TypeScript`, 8, 18);
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
canvas.width = W;
canvas.height = H;
requestAnimationFrame(frame);
