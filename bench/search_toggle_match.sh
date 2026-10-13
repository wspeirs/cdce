#!/usr/bin/env bash
# SPRT (plan.md §9.5): CDCE standard (engine1) vs LiTE (engine2) with one
# search toggle turned off on BOTH engines (symmetric).
#
# Usage: search_toggle_match.sh TAG TC HASH ELO0 ELO1 TOGGLE
#   TOGGLE: one of Search_AW Search_CheckExt Search_CorrHist Search_FP
#           Search_FutRed Search_HistPrune Search_IIR Search_LMP Search_LMR
#           Search_NMP Search_Razor Search_RFP Search_RFR Search_SE
#           Search_SEEPrune, or "none" for the all-on control.
#   e.g. search_toggle_match.sh sprt_toggle_nmp_off_stc "tc=8+0.08" 16 0 5 Search_NMP
# Engine1 CDCE: standard Val_* set via options. Engine2 LiTE: exposes no Val_*.
# Refuses to overwrite an existing record. Output: results/<TAG>.pgn / .log.
set -euo pipefail

TAG="${1:?TAG required}"
TC="${2:?time control required, e.g. tc=8+0.08}"
HASH="${3:?hash MB required}"
ELO0="${4:?elo0 required}"
ELO1="${5:?elo1 required}"
TOGGLE="${6:?toggle required: Search_* name or none}"
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

TOGGLE_OPTS=()
if [ "$TOGGLE" != "none" ]; then
  case "$TOGGLE" in
    Search_AW|Search_CheckExt|Search_CorrHist|Search_FP|Search_FutRed|Search_HistPrune|Search_IIR|Search_LMP|Search_LMR|Search_NMP|Search_Razor|Search_RFP|Search_RFR|Search_SE|Search_SEEPrune)
      TOGGLE_OPTS=(option."$TOGGLE"=false) ;;
    *) echo "unknown toggle: $TOGGLE" >&2; exit 1 ;;
  esac
fi

"$FASTCHESS" \
  -engine cmd="$ENGINE" name=CDCE option.SetEval=CDCE "${STD_OPTS[@]}" "${TOGGLE_OPTS[@]}" \
  -engine cmd="$ENGINE" name=LiTE option.SetEval=LiTE "${TOGGLE_OPTS[@]}" \
  -each $TC option.Hash="$HASH" option.Threads=1 option.Ponder=false \
  -openings file="$BOOK" format=epd order=random \
  -rounds 100000 -games 2 \
  -concurrency "$CONCURRENCY" \
  -sprt elo0="$ELO0" elo1="$ELO1" alpha=0.05 beta=0.05 \
  -pgnout file="$PGN" \
  2>&1 | tee "$OUT"
