// Particle Storm — Selvr emits struct-of-arrays (SoA) for hot sim loops:
// Float64Array xs, ys, vxs, vys — no per-frame allocation, cache-friendly.
//
// Idiomatic TypeScript often uses Particle[]; each step touches 48k heap objects.

fn main(): void {}
