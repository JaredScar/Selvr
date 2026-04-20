// Orb Catcher — Selvr implementation (same rules as the TypeScript version).
// Catch falling orbs with the paddle; miss eight and the round ends.
//
// Build when supported:  selvr build game.self
// Produces tree-shaken JS with sound types preserved through the pipeline.

import { Canvas, Context2D } from "std/web/canvas";
import { requestAnimationFrame } from "std/web/time";

const WIDTH: f64 = 560.0;
const HEIGHT: f64 = 420.0;
const PLAYER_W: f64 = 72.0;
const ORB_R: f64 = 10.0;
const GRAVITY: f64 = 0.18;

struct Orb {
    x: f64;
    y: f64;
    vy: f64;
    hue: f64;
    caught: boolean;
}

impl Orb {
    fn new(): Orb {
        return Orb {
            x: Math.random() * (WIDTH - ORB_R * 4.0) + ORB_R * 2.0,
            y: -ORB_R,
            vy: Math.random() * 1.6 + 1.2,
            hue: Math.random() * 360.0,
            caught: false,
        };
    }
}

struct Game {
    ctx: Context2D;
    orbs: Orb[];
    player_x: f64;
    score: i32;
    missed: i32;
    last_ts: f64;
    running: boolean;
}

impl Game {
    fn new(ctx: Context2D): Game {
        return Game {
            ctx,
            orbs: [],
            player_x: WIDTH / 2.0 - PLAYER_W / 2.0,
            score: 0,
            missed: 0,
            last_ts: 0.0,
            running: true,
        };
    }

    fn spawn(self): void {
        self.orbs.push(Orb.new());
    }

    fn tick(self, ts: f64): void {
        if !self.running {
            return;
        }
        const dt = if self.last_ts === 0.0 { 16.0 } else { Math.min(ts - self.last_ts, 48.0) };
        self.last_ts = ts;
        const t = dt / 16.67;

        if Math.random() < 0.02 * t {
            self.spawn();
        }

        for orb in self.orbs {
            if orb.caught {
                continue;
            }
            orb.vy += GRAVITY * t;
            orb.y += orb.vy * t;
            const px = self.player_x + PLAYER_W / 2.0;
            if orb.y + ORB_R >= HEIGHT - 28.0 && orb.y <= HEIGHT - 8.0
                && Math.abs(orb.x - px) < PLAYER_W / 2.0 + ORB_R
            {
                orb.caught = true;
                self.score += 1;
            } else if orb.y > HEIGHT + ORB_R {
                orb.caught = true;
                self.missed += 1;
                if self.missed >= 8 {
                    self.running = false;
                }
            }
        }

        self.draw();
        requestAnimationFrame((t2) => this.tick(t2));
    }

    fn draw(self): void {
        const g = self.ctx;
        g.fillStyle = "#0d1117";
        g.fillRect(0.0, 0.0, WIDTH, HEIGHT);
        g.fillStyle = "rgba(22, 27, 34, 0.9)";
        g.fillRect(0.0, 0.0, WIDTH, 44.0);
        g.fillStyle = "#8b949e";
        g.font = "13px ui-monospace, monospace";
        g.fillText(`Selvr → JS  ·  score ${self.score}  ·  misses ${self.missed}/8`, 12.0, 26.0);

        for orb in self.orbs {
            if orb.caught {
                continue;
            }
            g.beginPath();
            g.arc(orb.x, orb.y, ORB_R, 0.0, Math.PI * 2.0);
            g.fillStyle = `hsl(${orb.hue}, 75%, 58%)`;
            g.fill();
        }

        g.fillStyle = "#00d4aa";
        g.fillRect(self.player_x, HEIGHT - 24.0, PLAYER_W, 10.0);
        g.strokeStyle = "#30363d";
        g.strokeRect(0.0, 0.0, WIDTH, HEIGHT);

        if !self.running {
            g.fillStyle = "rgba(0,0,0,0.65)";
            g.fillRect(0.0, 0.0, WIDTH, HEIGHT);
            g.fillStyle = "#f85149";
            g.font = "bold 22px system-ui";
            g.textAlign = "center";
            g.fillText("Game over", WIDTH / 2.0, HEIGHT / 2.0 - 8.0);
            g.fillStyle = "#8b949e";
            g.font = "14px system-ui";
            g.fillText(`Final score: ${self.score}`, WIDTH / 2.0, HEIGHT / 2.0 + 18.0);
            g.textAlign = "left";
        }
    }
}

async fn main(): void {
    const canvas = Canvas.query("#game-canvas").unwrap();
    canvas.width = WIDTH as i32;
    canvas.height = HEIGHT as i32;
    const ctx = canvas.getContext("2d").unwrap();
    const game = Game.new(ctx);

    window.addEventListener("keydown", (e) => {
        const step = 14.0;
        if e.key === "ArrowLeft" || e.key === "a" {
            game.player_x = Math.max(0.0, game.player_x - step);
        }
        if e.key === "ArrowRight" || e.key === "d" {
            game.player_x = Math.min(WIDTH - PLAYER_W, game.player_x + step);
        }
    });

    requestAnimationFrame((ts) => game.tick(ts));
}
