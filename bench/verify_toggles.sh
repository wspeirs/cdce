#!/usr/bin/env bash
# Verify the CAPS search-toggle fork patch (plan.md §4 "Fork patch 1",
# M1 acceptance criteria, plan.md §10):
#   - with all toggles at their default (on), `bench` node count and hash
#     must match the pinned M0 baseline exactly;
#   - setting any single Search_* toggle to `false` must change the node
#     count (the technique actually fired during the baseline search).
#
# Usage: verify_toggles.sh [path-to-cdce-binary]
#   Default binary: <repo-root>/target/release/cdce
#
# Exit status: 0 if every check passes, 1 otherwise.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${1:-$REPO_ROOT/target/release/cdce}"

BASELINE_NODES=5681318
BASELINE_HASH="C08365ABC9AEA053"

TOGGLES=(
  Search_AW Search_CheckExt Search_RFP Search_RFR Search_Razor
  Search_NMP Search_IIR Search_LMP Search_FP Search_HistPrune
  Search_SEEPrune Search_SE Search_LMR Search_FutRed Search_CorrHist
)

if [[ ! -x "$BIN" ]]; then
  echo "cdce binary not found/executable at: $BIN" >&2
  echo "Build it first (cargo build --release -p cdce), then re-run." >&2
  exit 1
fi

# run_bench EXTRA_UCI_CMD -> prints the engine's stdout for one `bench` run.
run_bench() {
  "$BIN" <<EOF 2>&1
$1
bench
quit
EOF
}

parse_nodes() { grep -oP '\d+(?= nodes)' <<<"$1" | tail -1; }
parse_hash()  { grep -oP '(?<=hash )[0-9A-Fa-f]+' <<<"$1" | tail -1; }

PASS=0
FAIL=0
declare -a ROWS

check() {
  # check NAME NODES HASH OK_DESC FAIL_DESC OK
  local name=$1 nodes=$2 hash=$3 ok=$4 msg=$5
  if [[ "$ok" == 1 ]]; then
    ROWS+=("$(printf '%-18s %-10s %-18s %s' "$name" "$nodes" "$hash" "PASS $msg")")
    PASS=$((PASS + 1))
  else
    ROWS+=("$(printf '%-18s %-10s %-18s %s' "$name" "${nodes:--}" "${hash:--}" "FAIL $msg")")
    FAIL=$((FAIL + 1))
  fi
}

echo "Binary: $BIN"
echo "Running baseline bench..."
BASE_OUT=$(run_bench "")
BASE_NODES=$(parse_nodes "$BASE_OUT")
BASE_HASH=$(parse_hash "$BASE_OUT")

if [[ -z "$BASE_NODES" ]]; then
  echo "No 'bench' output parsed at all -- is this the cdce UCI binary?" >&2
  echo "$BASE_OUT" >&2
  exit 1
fi

if [[ "$BASE_NODES" == "$BASELINE_NODES" && "$BASE_HASH" == "$BASELINE_HASH" ]]; then
  check "baseline" "$BASE_NODES" "$BASE_HASH" 1 "(matches M0 baseline)"
else
  check "baseline" "$BASE_NODES" "$BASE_HASH" 0 \
    "(want $BASELINE_NODES nodes / $BASELINE_HASH hash)"
fi

for t in "${TOGGLES[@]}"; do
  echo "Running bench with $t=false..."
  OUT=$(run_bench "setoption name $t value false")
  NODES=$(parse_nodes "$OUT")
  HASH=$(parse_hash "$OUT")
  if [[ -z "$NODES" ]]; then
    check "$t" "" "" 0 "(no bench output -- option not registered?)"
  elif [[ "$NODES" != "$BASE_NODES" ]]; then
    check "$t" "$NODES" "$HASH" 1 "(node count changed)"
  else
    check "$t" "$NODES" "$HASH" 0 "(node count unchanged -- toggle had no effect)"
  fi
done

echo
printf '%-18s %-10s %-18s %s\n' "CHECK" "NODES" "HASH" "RESULT"
printf '%s\n' "----------------------------------------------------------------"
for row in "${ROWS[@]}"; do
  echo "$row"
done
echo
echo "$PASS passed, $FAIL failed (baseline + ${#TOGGLES[@]} toggles)"

[[ $FAIL -eq 0 ]]
