/**
 * Core Harvest — Selvr-aligned bundle (integer / BigInt mass only).
 * Replace with `selvr build game.self` when the toolchain compiles it.
 *
 * Selvr maps integer types to exact arithmetic — no silent IEEE-754 rounding
 * when accumulating large core mass.
 */
(() => {
  const MAX_SAFE = Number.MAX_SAFE_INTEGER;

  let mass = BigInt(MAX_SAFE - 2);
  let harvests = 0;

  const elMass = document.getElementById("mass-display");
  const elHarvests = document.getElementById("harvest-count");
  const btn = document.getElementById("harvest");
  if (!elMass || !elHarvests || !btn) throw new Error("missing DOM");

  function sync() {
    elMass.textContent = mass.toLocaleString();
    elHarvests.textContent = String(harvests);
  }

  function harvest() {
    mass += 1n;
    harvests += 1;
    sync();
  }

  btn.addEventListener("click", harvest);
  sync();
})();
