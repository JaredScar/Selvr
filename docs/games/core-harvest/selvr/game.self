// Core Harvest — Selvr sketch: `mass` as i64 (exact +1 past 2^53−1).
// Runnable demo: `game.js` (BigInt). Rebuild with `selvr build` when supported.
//
// The TypeScript version uses `number` and hits IEEE-754 integer limits; Selvr targets
// sound fixed-width integers for economy / scores without silent rounding.

fn main(): void {
    // UI: wire Harvest button to mass += 1 on i64.
}
