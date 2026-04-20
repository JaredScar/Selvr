/**
 * Particle Storm — idiomatic TypeScript: array of mutable particle objects.
 * Real games often look like this; every frame touches thousands of heap objects.
 */

const W = 560;
const H = 420;
const N = 48_000;

class Particle {
  x = 0;
  y = 0;
  vx = 0;
  vy = 0;
}

const particles: Particle[] = [];
for (let i = 0; i < N; i++) {
  const p = new Particle();
  p.x = Math.random() * W;
  p.y = Math.random() * H;
  p.vx = (Math.random() - 0.5) * 3.5;
  p.vy = (Math.random() - 0.5) * 3.5;
  particles.push(p);
}

const canvas = document.getElementById("game-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d");
if (!ctx) throw new Error("2d");

const elFps = document.getElementById("fps")!;
const elMs = document.getElementById("compute-ms")!;

let last = performance.now();
let frames = 0;
let fpsSmooth = 60;
let computeMsSmooth = 0;

const STRIDE = 24;

function frame(now: number): void {
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
  ctx.fillText(`${N.toLocaleString()} particles · heap objects · TypeScript`, 8, 18);

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

canvas.width = W;
canvas.height = H;
requestAnimationFrame(frame);
