#!/usr/bin/env bash
# Milestone 2 acceptance smoke test (plan.md §10, M2):
#   "a 1000-game fastchess run vs MateOnCE at STC with no crashes, illegal
#    moves, or time losses"
#
# Pits the CDCE eval against the MateOnCE eval, both running inside the same
# ./target/release/cdce UCI binary (SetEval switches the eval at runtime,
# plan.md §5), at STC with the standard bench settings (plan.md §9):
# Threads=1, Hash=16, Ponder=false, UHO_Lichess_4852_v1.epd book,
# color-reversed pairs.
#
# NOTE on initstr: fastchess has no `initstr=` engine option (that's a
# cutechess-cli concept -- checked the fastchess --help/man.md and source,
# no such key exists). The fastchess equivalent is `option.<Name>=<Value>`,
# which fastchess sends as `setoption name <Name> value <Value>` right after
# the engine starts, before any game begins -- exactly the semantics
# plan.md §5 requires ("GUIs and scripts must send SetEval before eval
# parameter options"). This script uses `option.SetEval=CDCE` /
# `option.SetEval=MateOnCE` in place of the requested initstr=.
#
# Pass/fail parsing: fastchess prints, per finished game,
# "Finished game N (...): <result> {<reason>}" where <reason> is exactly
# "<color> makes an illegal move" for an illegal move, or
# "<color> loses on time (<ms>ms overrun)" for a time forfeit; and at exit
# it prints, per player, "Player: <name>" / "Timeouts: N" / "Crashed: N"
# (crash count = engine disconnects/handshake failures). All three are
# parsed from the captured log; nothing here is a stdout eyeball check.
#
# Usage: smoke_m2_fastchess.sh [--smoke]
#   (default) 500 rounds x 2 games = 1000 games, full M2 acceptance run.
#   --smoke   5 rounds x 2 games = 10 games, quick wiring check.
#
# Exit status: 0 only if the run finished all expected games with 0
# crashes, 0 illegal moves, and 0 time losses. Writes
# results/m2_cdce_vs_mateonce.pgn and results/m2_cdce_vs_mateonce.log.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FASTCHESS="$REPO_ROOT/bench/bin/fastchess"
BOOK="$REPO_ROOT/bench/UHO_Lichess_4852_v1.epd"
RESULTS_DIR="$REPO_ROOT/results"
CDCE_BIN="$REPO_ROOT/target/release/cdce"
TAG="${TAG:-m2_cdce_vs_mateonce}"
# CDCE_EVAL: SetEval name for the CDCE side (default CDCE; e.g. CDCE-Core, CDCE-Material).
# CDCE_OPTS: extra space-separated UCI options for the CDCE side, sent after SetEval,
#   e.g. "option.Val_Knight=320 option.Val_Rook=500 option.Val_Queen=900" (the `standard` set).
# The M2 record used the pseudocode defaults, so no extra options are needed to reproduce it.
CDCE_EVAL="${CDCE_EVAL:-CDCE}"
CDCE_OPTS="${CDCE_OPTS:-}"
CONCURRENCY="${CONCURRENCY:-$(nproc)}"

ROUNDS="${ROUNDS:-500}"
if [[ "${1:-}" == "--smoke" ]]; then
  ROUNDS=5
fi

if [[ ! -x "$CDCE_BIN" ]]; then
  echo "cdce binary not found/executable at: $CDCE_BIN" >&2
  echo "Build it first (cargo build --release -p cdce), then re-run." >&2
  exit 1
fi

mkdir -p "$RESULTS_DIR"
PGN="$RESULTS_DIR/${TAG}.pgn"
LOG="$RESULTS_DIR/${TAG}.log"
if [[ -e "$PGN" || -e "$LOG" ]]; then
  if [[ "${FORCE:-0}" != "1" ]]; then
    echo "Refusing to overwrite existing record: $PGN / $LOG (set FORCE=1 or use a new TAG)." >&2
    exit 1
  fi
  rm -f "$PGN" "$LOG"
fi

cd "$REPO_ROOT"
"$FASTCHESS" \
  -engine cmd=./target/release/cdce name=CDCE option.SetEval="$CDCE_EVAL" $CDCE_OPTS \
  -engine cmd=./target/release/cdce name=MateOnCE option.SetEval=MateOnCE \
  -each tc=8+0.08 option.Hash=16 option.Threads=1 option.Ponder=false \
  -openings file="$BOOK" format=epd order=random \
  -rounds "$ROUNDS" -games 2 \
  -concurrency "$CONCURRENCY" \
  -pgnout file="$PGN" \
  -ratinginterval 10 \
  2>&1 | tee "$LOG"

EXPECTED_GAMES=$((ROUNDS * 2))

ILLEGAL=$(grep -c "makes an illegal move" "$LOG" || true)
GAMES_PLAYED=$(grep -c "^Finished game " "$LOG" || true)
CRASHED=$( (grep -oP '(?<=Crashed: )\d+' "$LOG" || true) | awk '{s+=$1} END {print s+0}')
TIMEOUTS=$( (grep -oP '(?<=Timeouts: )\d+' "$LOG" || true) | awk '{s+=$1} END {print s+0}')

echo
echo "== M2 smoke test summary ($TAG) =="
echo "Games played:  $GAMES_PLAYED / $EXPECTED_GAMES"
echo "Illegal moves: $ILLEGAL"
echo "Time losses:   $TIMEOUTS"
echo "Crashes:       $CRASHED"
echo "PGN: $PGN"
echo "Log: $LOG"

FAIL=0
[[ "$GAMES_PLAYED" -eq "$EXPECTED_GAMES" ]] || { echo "FAIL: expected $EXPECTED_GAMES games, only $GAMES_PLAYED finished"; FAIL=1; }
[[ "$ILLEGAL" -eq 0 ]]  || { echo "FAIL: $ILLEGAL illegal move(s)"; FAIL=1; }
[[ "$TIMEOUTS" -eq 0 ]] || { echo "FAIL: $TIMEOUTS time loss(es)"; FAIL=1; }
[[ "$CRASHED" -eq 0 ]]  || { echo "FAIL: $CRASHED crash(es)"; FAIL=1; }

if [[ $FAIL -eq 0 ]]; then
  echo "PASS: M2 acceptance criteria met."
fi
exit $FAIL
