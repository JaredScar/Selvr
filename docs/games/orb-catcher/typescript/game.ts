/**
 * Orb Catcher — TypeScript reference implementation.
 * Same rules as the Selvr version; compare bundle shape and runtime checks.
 */

const WIDTH = 560;
const HEIGHT = 420;
const PLAYER_W = 72;
const ORB_R = 10;
const GRAVITY = 0.18;

type Orb = { x: number; y: number; vy: number; hue: number; caught: boolean };

let canvas: HTMLCanvasElement;
let ctx: CanvasRenderingContext2D;
let playerX: number;
let score = 0;
let missed = 0;
let orbs: Orb[] = [];
let lastTs = 0;
let running = true;

function rand(min: number, max: number): number {
  return Math.random() * (max - min) + min;
}

function spawnOrb(): void {
  orbs.push({
    x: rand(ORB_R * 2, WIDTH - ORB_R * 2),
    y: -ORB_R,
    vy: rand(1.2, 2.8),
    hue: rand(0, 360),
    caught: false,
  });
}

function step(ts: number): void {
  if (!running) return;
  const dt = lastTs === 0 ? 16 : Math.min(ts - lastTs, 48);
  lastTs = ts;
  const t = dt / 16.67;

  if (Math.random() < 0.02 * t) spawnOrb();

  for (const o of orbs) {
    if (o.caught) continue;
    o.vy += GRAVITY * t;
    o.y += o.vy * t;
    const px = playerX + PLAYER_W / 2;
    if (
      o.y + ORB_R >= HEIGHT - 28 &&
      o.y <= HEIGHT - 8 &&
      Math.abs(o.x - px) < PLAYER_W / 2 + ORB_R
    ) {
      o.caught = true;
      score += 1;
    } else if (o.y > HEIGHT + ORB_R) {
      o.caught = true;
      missed += 1;
      if (missed >= 8) running = false;
    }
  }
  orbs = orbs.filter((o) => o.y < HEIGHT + 40);

  draw();
  requestAnimationFrame(step);
}

function draw(): void {
  const g = ctx;
  g.fillStyle = "#0d1117";
  g.fillRect(0, 0, WIDTH, HEIGHT);
  g.fillStyle = "rgba(22, 27, 34, 0.9)";
  g.fillRect(0, 0, WIDTH, 44);
  g.fillStyle = "#8b949e";
  g.font = "13px ui-monospace, monospace";
  g.fillText(`TypeScript build  ·  score ${score}  ·  misses ${missed}/8`, 12, 26);

  for (const o of orbs) {
    if (o.caught) continue;
    g.beginPath();
    g.arc(o.x, o.y, ORB_R, 0, Math.PI * 2);
    g.fillStyle = `hsl(${o.hue}, 75%, 58%)`;
    g.fill();
  }

  g.fillStyle = "#58a6ff";
  g.fillRect(playerX, HEIGHT - 24, PLAYER_W, 10);
  g.strokeStyle = "#30363d";
  g.strokeRect(0, 0, WIDTH, HEIGHT);

  if (!running) {
    g.fillStyle = "rgba(0,0,0,0.65)";
    g.fillRect(0, 0, WIDTH, HEIGHT);
    g.fillStyle = "#f85149";
    g.font = "bold 22px system-ui";
    g.textAlign = "center";
    g.fillText("Game over", WIDTH / 2, HEIGHT / 2 - 8);
    g.fillStyle = "#8b949e";
    g.font = "14px system-ui";
    g.fillText(`Final score: ${score}`, WIDTH / 2, HEIGHT / 2 + 18);
    g.textAlign = "left";
  }
}

function onKey(e: KeyboardEvent): void {
  const step = 14;
  if (e.key === "ArrowLeft" || e.key === "a") playerX = Math.max(0, playerX - step);
  if (e.key === "ArrowRight" || e.key === "d")
    playerX = Math.min(WIDTH - PLAYER_W, playerX + step);
}

function boot(): void {
  canvas = document.getElementById("game-canvas") as HTMLCanvasElement;
  const c = canvas.getContext("2d");
  if (!c) throw new Error("2d context unavailable");
  ctx = c;
  canvas.width = WIDTH;
  canvas.height = HEIGHT;
  playerX = WIDTH / 2 - PLAYER_W / 2;
  window.addEventListener("keydown", onKey);
  requestAnimationFrame(step);
}

boot();
