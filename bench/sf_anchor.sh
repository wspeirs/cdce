#!/usr/bin/env bash
# Absolute anchor (plan.md §9.4): CDCE standard vs Stockfish with UCI_LimitStrength.
# Reported as "SF-Elo@TC on this machine", not CCRL Elo.
#
# Usage: sf_anchor.sh TAG TC HASH SF_ELO ROUNDS
#   e.g. sf_anchor.sh sf2000_pilot_stc "tc=8+0.08" 16 2000 20    (20 rounds = 40 games)
# Both engines: Threads=1, Ponder=false, same Hash. CDCE: standard Val_* set.
# Stockfish: UCI_LimitStrength=true, UCI_Elo=SF_ELO. Refuses to overwrite a record.
# Output: results/<TAG>.pgn / .log. No SPRT: fixed-game match.
set -euo pipefail

TAG="${1:?TAG required}"
TC="${2:?time control required, e.g. tc=8+0.08}"
HASH="${3:?hash MB required}"
SF_ELO="${4:?UCI_Elo required (1320-3190 for this Stockfish build)}"
ROUNDS="${5:?rounds required (games = 2 * rounds)}"
if ! [[ "$SF_ELO" =~ ^[0-9]+$ ]] || [ "$SF_ELO" -lt 1320 ] || [ "$SF_ELO" -gt 3190 ]; then
  echo "UCI_Elo $SF_ELO outside 1320-3190 for this Stockfish build" >&2
  exit 1
fi
BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$BENCH_DIR/.."
FASTCHESS="$BENCH_DIR/bin/fastchess"
BOOK="$BENCH_DIR/UHO_Lichess_4852_v1.epd"
ENGINE="$ROOT/target/release/cdce"
STOCKFISH="${STOCKFISH:-/usr/games/stockfish}"
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
  -engine cmd="$STOCKFISH" name="SF-Elo${SF_ELO}" option.UCI_LimitStrength=true option.UCI_Elo="$SF_ELO" \
  -each $TC option.Hash="$HASH" option.Threads=1 option.Ponder=false \
  -openings file="$BOOK" format=epd order=random \
  -rounds "$ROUNDS" -games 2 \
  -concurrency "$CONCURRENCY" \
  -pgnout file="$PGN" \
  2>&1 | tee "$OUT"
