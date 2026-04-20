// 24 — GC-pause benchmark: animation loop under memory pressure
// This is SELVR's core differentiation demo vs JavaScript.
//
// Run the equivalent JS/TS version side-by-side to see jank vs. smoothness.
//
// Why SELVR wins:
//   - Every Particle is freed deterministically when it goes out of scope
//   - JavaScript's GC decides when to collect, causing unpredictable frame drops
//   - SELVR's ownership model = predictable 60fps even under allocation pressure

import { Canvas, Context2D } from "std/web/canvas";
import { requestAnimationFrame, now } from "std/web/time";
import { random } from "std/math";

const PARTICLE_COUNT: i32 = 5000;
const WIDTH:  f64 = 1280.0;
const HEIGHT: f64 = 720.0;

struct Particle {
    x: f64;
    y: f64;
    vx: f64;
    vy: f64;
    life: f64;   // 1.0 = fresh, 0.0 = dead
    size: f64;
    hue: f64;
}

impl Particle {
    fn random(): Particle {
        return Particle {
            x:    WIDTH  * random(),
            y:    HEIGHT * random(),
            vx:   (random() - 0.5) * 200.0,
            vy:   (random() - 0.5) * 200.0,
            life: random(),
            size: 2.0 + random() * 4.0,
            hue:  random() * 360.0,
        };
    }

    // Returns false when the particle is dead and should be replaced
    fn update(dt: f64): boolean {
        this.x    += this.vx * dt;
        this.y    += this.vy * dt;
        this.life -= dt * 0.5;
        return this.life > 0.0;
    }

    fn draw(ctx: Context2D): void {
        ctx.globalAlpha = this.life;
        ctx.beginPath();
        ctx.arc(this.x, this.y, this.size, 0.0, Math.PI * 2.0);
        ctx.fillStyle = `hsl(${this.hue.toFixed(0)}, 100%, 60%)`;
        ctx.fill();
    }
}

struct Benchmark {
    ctx: Context2D;
    particles: Particle[];
    frameTimes: f64[];
    lastTime: f64;
}

impl Benchmark {
    fn new(ctx: Context2D): Benchmark {
        const particles = (0..PARTICLE_COUNT).map((_) => Particle.random()).toArray();
        return Benchmark { ctx, particles, frameTimes: [], lastTime: 0.0 };
    }

    fn tick(timestamp: f64): void {
        const dt = if this.lastTime === 0.0 { 0.016 } else {
            Math.min((timestamp - this.lastTime) / 1000.0, 0.05)
        };
        this.lastTime = timestamp;
        this.frameTimes.push(dt * 1000.0);
        if this.frameTimes.length > 120 { this.frameTimes.shift(); }

        // Clear with motion blur
        this.ctx.globalAlpha = 1.0;
        this.ctx.fillStyle = "rgba(10, 10, 20, 0.3)";
        this.ctx.fillRect(0.0, 0.0, WIDTH, HEIGHT);

        // Update — dead particles replaced with fresh ones.
        // In SELVR: the old Particle is freed immediately, deterministically.
        // In JavaScript: the GC decides when (if ever) it collects the old object.
        this.particles = this.particles.map((p) => {
            if p.update(dt) { return p; }
            return Particle.random();  // old Particle freed here
        });

        for p in this.particles { p.draw(this.ctx); }

        // HUD
        if this.frameTimes.length > 0 {
            const avgMs = this.frameTimes.reduce((a, b) => a + b, 0.0) / this.frameTimes.length as f64;
            const maxMs = this.frameTimes.reduce((a, b) => Math.max(a, b), 0.0);
            const fps   = 1000.0 / avgMs;

            this.ctx.globalAlpha = 1.0;
            this.ctx.fillStyle = "#00ff88";
            this.ctx.font = "bold 14px monospace";
            this.ctx.fillText(
                `FPS: ${fps.toFixed(1)}  avg: ${avgMs.toFixed(1)}ms  max: ${maxMs.toFixed(1)}ms  particles: ${PARTICLE_COUNT}`,
                10.0, 20.0
            );
        }

        requestAnimationFrame((ts) => this.tick(ts));
    }
}

async fn main(): void {
    const canvas = Canvas.query("#bench-canvas").unwrap();
    canvas.width  = WIDTH as i32;
    canvas.height = HEIGHT as i32;
    const ctx = canvas.getContext("2d").unwrap();

    const bench = Benchmark.new(ctx);
    requestAnimationFrame((ts) => bench.tick(ts));
}
