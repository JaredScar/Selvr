/**
 * Core Harvest — TypeScript version
 *
 * Uses `number` for displayed mass (what most games do for "simplicity").
 * IEEE-754 doubles cannot represent every integer above 2^53 − 1; increments of 1
 * stop working — a real correctness bug that `number` types do not prevent.
 */

const MAX_SAFE = Number.MAX_SAFE_INTEGER; // 9007199254740991

let massNumber = MAX_SAFE - 2; // two increments before the wall
let harvests = 0;

/** Ground truth — what the total *should* be if every harvest added exactly +1. */
let massTruth = BigInt(MAX_SAFE - 2);

const elMass = document.getElementById("mass-display")!;
const elTruth = document.getElementById("truth-display")!;
const elWarn = document.getElementById("precision-warning")!;
const elHarvests = document.getElementById("harvest-count")!;
const btn = document.getElementById("harvest")!;

function sync(): void {
  elMass.textContent = massNumber.toLocaleString();
  elTruth.textContent = massTruth.toLocaleString();
  elHarvests.textContent = String(harvests);
  const drift = BigInt(Math.trunc(massNumber)) !== massTruth;
  elWarn.hidden = !drift;
}

function harvest(): void {
  massNumber += 1;
  massTruth += 1n;
  harvests += 1;
  sync();
}

btn.addEventListener("click", harvest);
sync();
