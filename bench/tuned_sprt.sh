#!/usr/bin/env bash
# SPRT (plan.md §9): CDCE-Tuned (engine1, H1 side) vs CDCE standard (engine2).
#
# Engine1 CDCE-Tuned: fitted weights from TunedValues::tuned(); no Val_* options exist for it.
# Engine2 CDCE: CdceParams standard set via Val_* options (same set as the M5 records).
# Tempo: both use CdceParams::default().tempo (TunedValues takes it from there; no Tempo UCI
# option is exposed on the tuned side, so Tempo is not passed to either engine).
#
# Usage: tuned_sprt.sh [tag] [tc] [hash]
#   defaults: tag m4_tuned_vs_cdce_stc, tc 8+0.08, hash 16
#   LTC example: tuned_sprt.sh m4_tuned_vs_cdce_ltc 40+0.4 64
# Refuses to overwrite an existing record. Output: results/<tag>.pgn / .log.
set -euo pipefail

TAG="${1:-m4_tuned_vs_cdce_stc}"
TC="${2:-8+0.08}"
HASH="${3:-16}"
BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$BENCH_DIR/.."
FASTCHESS="$BENCH_DIR/bin/fastchess"
BOOK="$BENCH_DIR/UHO_Lichess_4852_v1.epd"
ENGINE="$ROOT/target/release/cdce"
RESULTS_DIR="$ROOT/results"
CONCURRENCY="${CONCURRENCY:-$(nproc)}"
OUT="$RESULTS_DIR/${TAG}.log"
PGN="$RESULTS_DIR/${TAG}.pgn"

if [ -e "$OUT" ] || [ -e "$PGN" ]; then
  echo "refusing to overwrite existing record for TAG=$TAG" >&2
  exit 1
fi
mkdir -p "$RESULTS_DIR"

STD_OPTS=(option.Val_Pawn=100 option.Val_Knight=320 option.Val_Bishop=330 option.Val_Rook=500 option.Val_Queen=900)

"$FASTCHESS" \
  -engine cmd="$ENGINE" name=CDCE-Tuned option.SetEval=CDCE-Tuned \
  -engine cmd="$ENGINE" name=CDCE option.SetEval=CDCE "${STD_OPTS[@]}" \
  -each tc="$TC" option.Hash="$HASH" option.Threads=1 option.Ponder=false \
  -openings file="$BOOK" format=epd order=random \
  -rounds 100000 -games 2 \
  -concurrency "$CONCURRENCY" \
  -sprt elo0=0 elo1=5 alpha=0.05 beta=0.05 \
  -pgnout file="$PGN" \
  2>&1 | tee "$OUT"
