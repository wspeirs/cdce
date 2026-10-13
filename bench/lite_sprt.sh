#!/usr/bin/env bash
# SPRT (plan.md §9.3): CDCE standard (engine1, H1 side) vs LiTE (engine2).
# Engine1 CDCE: standard Val_* set via options. Engine2 LiTE: exposes no Val_* options.
#
# Usage: lite_sprt.sh TAG TC HASH ELO0 ELO1
#   e.g. lite_sprt.sh sprt_cdce_std_vs_lite_stc_0_5 "tc=8+0.08" 16 0 5
# Refuses to overwrite an existing record. Output: results/<TAG>.pgn / .log.
set -euo pipefail

TAG="${1:?TAG required}"
TC="${2:?time control required, e.g. tc=8+0.08}"
HASH="${3:?hash MB required}"
ELO0="${4:?elo0 required}"
ELO1="${5:?elo1 required}"
BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$BENCH_DIR/.."
FASTCHESS="$BENCH_DIR/bin/fastchess"
BOOK="$BENCH_DIR/UHO_Lichess_4852_v1.epd"
ENGINE="$ROOT/target/release/cdce"
RESULTS_DIR="$ROOT/results"
CONCURRENCY="${CONCURRENCY:-7}"
OUT="$RESULTS_DIR/${TAG}.log"
PGN="$RESULTS_DIR/${TAG}.pgn"

if [ -e "$OUT" ] || [ -e "$PGN" ]; then
  echo "refusing to overwrite existing record for TAG=$TAG" >&2
  exit 1
fi
mkdir -p "$RESULTS_DIR"

STD_OPTS=(option.Val_Pawn=100 option.Val_Knight=320 option.Val_Bishop=330 option.Val_Rook=500 option.Val_Queen=900)

"$FASTCHESS" \
  -engine cmd="$ENGINE" name=CDCE option.SetEval=CDCE "${STD_OPTS[@]}" \
  -engine cmd="$ENGINE" name=LiTE option.SetEval=LiTE \
  -each $TC option.Hash="$HASH" option.Threads=1 option.Ponder=false \
  -openings file="$BOOK" format=epd order=random \
  -rounds 100000 -games 2 \
  -concurrency "$CONCURRENCY" \
  -sprt elo0="$ELO0" elo1="$ELO1" alpha=0.05 beta=0.05 \
  -pgnout file="$PGN" \
  2>&1 | tee "$OUT"
