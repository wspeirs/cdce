#!/usr/bin/env bash
# Reduced CCRL-anchor ladder (plan.md §9.4b): CDCE standard vs one CCRL-listed opponent.
# Fixed-game match, not an SPRT. Refuses to overwrite a record.
#
# Usage: ladder_match.sh TAG OPP_NAME OPP_CMD ROUNDS
#   e.g. ladder_match.sh ccrl_stash37_stc stash37 /tmp/stash-bot/src/stash 100   (100 rounds = 200 games)
# Both engines: Threads=1, Ponder=false, Hash 16, UHO book with color-reversed pairs, tc=8+0.08.
# CDCE: standard Val_* set. Opponent: no Val_* (it is not a CDCE build).
# Output: results/<TAG>.log / .pgn. The CCRL rating of OPP_NAME is from the CCRL 40/15 list
# (4-CPU, 40/15), which is not the time control used here; report implied ratings as relative only.
set -euo pipefail

TAG="${1:?TAG required}"
OPP_NAME="${2:?opponent name required}"
OPP_CMD="${3:?opponent binary path required}"
ROUNDS="${4:?rounds required (games = 2 * rounds)}"
TC="tc=8+0.08"
HASH=16
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
[ -x "$OPP_CMD" ] || { echo "opponent binary not executable: $OPP_CMD" >&2; exit 1; }
mkdir -p "$RESULTS_DIR"

STD_OPTS=(option.Val_Pawn=100 option.Val_Knight=320 option.Val_Bishop=330 option.Val_Rook=500 option.Val_Queen=900)

# Opponent options: Threads=1 and Ponder=false by default. OPP_NO_THREADS=1 / OPP_NO_PONDER=1 drop them
# for engines that do not advertise the option (GreKo: no Ponder; Coiled 1.1: no Threads, no Ponder).
OPP_OPTS=()
[ "${OPP_NO_THREADS:-0}" = 1 ] || OPP_OPTS+=(option.Threads=1)
[ "${OPP_NO_PONDER:-0}" = 1 ] || OPP_OPTS+=(option.Ponder=false)

"$FASTCHESS" \
  -engine cmd="$ENGINE" name=CDCE option.SetEval=CDCE "${STD_OPTS[@]}" option.Threads=1 option.Ponder=false \
  -engine cmd="$OPP_CMD" name="$OPP_NAME" "${OPP_OPTS[@]}" \
  -each $TC option.Hash="$HASH" \
  -openings file="$BOOK" format=epd order=random \
  -rounds "$ROUNDS" -games 2 \
  -concurrency "$CONCURRENCY" \
  -pgnout file="$PGN" \
  2>&1 | tee "$OUT"
