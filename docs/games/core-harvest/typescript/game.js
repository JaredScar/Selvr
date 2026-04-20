// game.ts
var MAX_SAFE = Number.MAX_SAFE_INTEGER;
var massNumber = MAX_SAFE - 2;
var harvests = 0;
var massTruth = BigInt(MAX_SAFE - 2);
var elMass = document.getElementById("mass-display");
var elTruth = document.getElementById("truth-display");
var elWarn = document.getElementById("precision-warning");
var elHarvests = document.getElementById("harvest-count");
var btn = document.getElementById("harvest");
function sync() {
  elMass.textContent = massNumber.toLocaleString();
  elTruth.textContent = massTruth.toLocaleString();
  elHarvests.textContent = String(harvests);
  const drift = BigInt(Math.trunc(massNumber)) !== massTruth;
  elWarn.hidden = !drift;
}
function harvest() {
  massNumber += 1;
  massTruth += 1n;
  harvests += 1;
  sync();
}
btn.addEventListener("click", harvest);
sync();
