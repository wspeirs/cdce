#!/usr/bin/env bash
# Head-to-head ablation: <VARIANT> vs full CDCE, both on the standard value set
# (P100 N320 B330 R500 Q900) and bench settings (8+0.08, Hash=16, Threads=1,
# Ponder=false, UHO book, color-reversed pairs). Fixed rounds, no SPRT.
#
# Usage: ablation_match.sh <CDCE-variant-name> [rounds] [suffix]
#   rounds default 200 (= 400 games). suffix (optional) is appended to the record
#   name, e.g. `rerun`, so an earlier record is never overwritten.
# Refuses to overwrite an existing record (results/ablation/<TAG>.{pgn,log}).
set -euo pipefail

VARIANT=${1:?usage: ablation_match.sh <CDCE-variant-name> [rounds]}
ROUNDS=${2:-200}

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FASTCHESS="$REPO_ROOT/bench/bin/fastchess"
BOOK="$REPO_ROOT/bench/UHO_Lichess_4852_v1.epd"
RESULTS_DIR="$REPO_ROOT/results/ablation"
CONCURRENCY="${CONCURRENCY:-$(nproc)}"
STD="option.Val_Pawn=100 option.Val_Knight=320 option.Val_Bishop=330 option.Val_Rook=500 option.Val_Queen=900"

TAG="${VARIANT}_vs_CDCE_std${3:+_$3}"
mkdir -p "$RESULTS_DIR"
PGN="$RESULTS_DIR/${TAG}.pgn"
LOG="$RESULTS_DIR/${TAG}.log"
if [[ -e "$PGN" || -e "$LOG" ]]; then
  echo "Refusing to overwrite existing record: $PGN / $LOG" >&2
  exit 1
fi

cd "$REPO_ROOT"
# shellcheck disable=SC2086  # $STD is a deliberate word-split list of UCI options
"$FASTCHESS" \
  -engine cmd=./target/release/cdce name="$VARIANT" option.SetEval="$VARIANT" $STD \
  -engine cmd=./target/release/cdce name=CDCE option.SetEval=CDCE $STD \
  -each tc=8+0.08 option.Hash=16 option.Threads=1 option.Ponder=false \
  -openings file="$BOOK" format=epd order=random \
  -rounds "$ROUNDS" -games 2 \
  -concurrency "$CONCURRENCY" \
  -pgnout file="$PGN" \
  -ratinginterval 10 \
  2>&1 | tee "$LOG"
