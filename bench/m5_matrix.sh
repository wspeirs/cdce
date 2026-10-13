#!/usr/bin/env bash
# M5 matches (plan.md §9): CDCE standard vs LiTE, run sequentially so the matches do not share CPU.
#   1. depth=8 gauntlet, STC 8+0.08
#   2. nodes=20000 gauntlet, STC 8+0.08
#   3. LTC 40+0.4, Hash=64, fixed games
# These are fixed-game matches, not SPRTs. A full SPRT at LTC would take days on this machine.
# CDCE-Tuned LTC is NOT in this script: it waits on the tuner standard-weights holdout check.
# Usage: m5_matrix.sh. Refuses to overwrite any existing record.
set -euo pipefail

BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$BENCH_DIR/.."
FASTCHESS="$BENCH_DIR/bin/fastchess"
BOOK="$BENCH_DIR/UHO_Lichess_4852_v1.epd"
ENGINE="$ROOT/target/release/cdce"
RESULTS_DIR="$ROOT/results"
CONCURRENCY="${CONCURRENCY:-7}"
STD_OPTS=(option.Val_Pawn=100 option.Val_Knight=320 option.Val_Bishop=330 option.Val_Rook=500 option.Val_Queen=900)

run_match() {
  local tag="$1" games="$2" tc_args="$3" extra_hash="$4"; shift 4
  local out="$RESULTS_DIR/${tag}.log" pgn="$RESULTS_DIR/${tag}.pgn"
  if [ -e "$out" ] || [ -e "$pgn" ]; then
    echo "refusing to overwrite existing record for TAG=$tag" >&2
    return 1
  fi
  "$FASTCHESS" \
    -engine cmd="$ENGINE" name=CDCE option.SetEval=CDCE "${STD_OPTS[@]}" \
    -engine cmd="$ENGINE" name=LiTE option.SetEval=LiTE \
    -each $tc_args option.Hash="$extra_hash" option.Threads=1 option.Ponder=false \
    -openings file="$BOOK" format=epd order=random \
    -rounds "$((games / 2))" -games 2 \
    -concurrency "$CONCURRENCY" \
    -pgnout file="$pgn" \
    2>&1 | tee "$out"
}

mkdir -p "$RESULTS_DIR"
run_match m5_cdce_std_vs_lite_depth8_stc 400 "tc=8+0.08 depth=8" 16
run_match m5_cdce_std_vs_lite_nodes20k_stc 400 "tc=8+0.08 nodes=20000" 16
run_match m5_cdce_std_vs_lite_ltc 200 "tc=40+0.4" 64
