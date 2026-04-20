// 20 — Canvas game loop
// A bouncing-ball demo demonstrating the browser animation loop.
// For TypeScript developers: requestAnimationFrame, canvas 2D context — all familiar.

import { Canvas, Context2D } from "std/web/canvas";
import { requestAnimationFrame } from "std/web/time";

const WIDTH:  f64 = 800.0;
const HEIGHT: f64 = 600.0;
const RADIUS: f64 = 20.0;

struct Ball {
    x: f64;
    y: f64;
    vx: f64;
    vy: f64;
    color: string;
}

impl Ball {
    fn new(x: f64, y: f64, vx: f64, vy: f64, color: string): Ball {
        return Ball { x, y, vx, vy, color };
    }

    fn update(dt: f64): void {
        this.x += this.vx * dt;
        this.y += this.vy * dt;

        if this.x - RADIUS < 0.0 || this.x + RADIUS > WIDTH {
            this.vx = -this.vx;
            this.x  = Math.clamp(this.x, RADIUS, WIDTH - RADIUS);
        }
        if this.y - RADIUS < 0.0 || this.y + RADIUS > HEIGHT {
            this.vy = -this.vy;
            this.y  = Math.clamp(this.y, RADIUS, HEIGHT - RADIUS);
        }
    }

    fn draw(ctx: Context2D): void {
        ctx.beginPath();
        ctx.arc(this.x, this.y, RADIUS, 0.0, Math.PI * 2.0);
        ctx.fillStyle = this.color;
        ctx.fill();
    }
}

struct Game {
    balls: Ball[];
    ctx: Context2D;
    lastTime: f64;
}

impl Game {
    fn new(ctx: Context2D): Game {
        return Game {
            balls: [
                Ball.new(200.0, 150.0,  250.0,  180.0, "#e74c3c"),
                Ball.new(400.0, 300.0, -200.0,  220.0, "#3498db"),
                Ball.new(600.0, 450.0,  150.0, -190.0, "#2ecc71"),
                Ball.new(300.0, 200.0, -170.0, -210.0, "#f39c12"),
            ],
            ctx,
            lastTime: 0.0,
        };
    }

    fn tick(timestamp: f64): void {
        const dt = if this.lastTime === 0.0 { 0.016 } else {
            Math.min((timestamp - this.lastTime) / 1000.0, 0.05)
        };
        this.lastTime = timestamp;

        // Clear with a trail effect
        this.ctx.fillStyle = "rgba(10, 10, 20, 0.3)";
        this.ctx.fillRect(0.0, 0.0, WIDTH, HEIGHT);

        for ball in this.balls {
            ball.update(dt);
            ball.draw(this.ctx);
        }

        // HUD
        this.ctx.fillStyle = "#00ff88";
        this.ctx.font = "bold 14px monospace";
        this.ctx.fillText(`${this.balls.length} balls`, 10.0, 20.0);

        requestAnimationFrame((ts) => this.tick(ts));
    }
}

async fn main(): void {
    const canvas = Canvas.query("#game-canvas").unwrap();
    canvas.width  = WIDTH as i32;
    canvas.height = HEIGHT as i32;

    const ctx = canvas.getContext("2d").unwrap();
    const game = Game.new(ctx);

    requestAnimationFrame((ts) => game.tick(ts));
}
