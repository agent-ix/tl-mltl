#!/bin/sh
# Run one Kani harness by its exact qualified name and fail unless it verified
# with every cover property satisfied. Kani itself exits 0 when a cover is
# unsatisfied (a vacuous proof), so the gate reads the result line.
set -eu
harness=$1
out=$(mktemp)
trap 'rm -f "$out"' EXIT
cargo kani --lib -Z stubbing --output-format terse --harness "$harness" --exact >"$out" 2>&1 \
    || { cat "$out"; echo "kani-gate: $harness did not verify" >&2; exit 1; }
cat "$out"
grep -q 'VERIFICATION:- SUCCESSFUL' "$out" \
    || { echo "kani-gate: $harness has no successful verification" >&2; exit 1; }
cover=$(grep -E '\*\* [0-9]+ of [0-9]+ cover properties satisfied' "$out" || true)
[ -n "$cover" ] || { echo "kani-gate: $harness has no cover results (vacuous)" >&2; exit 1; }
echo "$cover" | awk '{ if ($2 != $4 || $4 == 0) exit 1 }' \
    || { echo "kani-gate: $harness has an unsatisfied cover: $cover" >&2; exit 1; }
