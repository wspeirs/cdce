#!/usr/bin/env bash
# Run a fastchess match/SPRT between two UCI engines with CDCE's standard
# bench settings (plan.md §9): Threads=1, Hash=16 (STC)/64 (LTC), Ponder=false,
# UHO_Lichess_4852_v1.epd opening book, color-reversed pairs (-games 2, i.e.
# each opening is played once per color).
#
# Usage: run_match.sh <stc|ltc> <engine1_cmd> <engine2_cmd> [rounds] [sprt]
#   tc      stc = 8+0.08, Hash=16 ; ltc = 40+0.4, Hash=64 (plan.md §9)
#   rounds  max rounds (each round = 2 games, same opening, both colors)
#           default: 100000 for sprt modes (SPRT stops itself), 100 for none
#   sprt    better (default) = SPRT [0,5] elo bound, alpha=beta=0.05
#           nonreg           = SPRT [-5,0] elo bound, alpha=beta=0.05
#           none              = fixed-round match, no SPRT
#
# Output: results/<tag>.pgn, results/<tag>.log, and a row appended to
# results/summary.csv. The nps/avg_depth columns are left blank here --
# fill them from a `bench`/criterion run (plan.md §9 point 1); this script
# only measures Elo/LOS/games from the match itself.
set -euo pipefail

TC=${1:?usage: run_match.sh <stc|ltc> engine1_cmd engine2_cmd [rounds] [sprt]}
E1=${2:?engine1 cmd required}
E2=${3:?engine2 cmd required}
SPRT=${5:-better}
ROUNDS=${4:-$([ "$SPRT" = none ] && echo 100 || echo 100000)}

BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FASTCHESS="$BENCH_DIR/bin/fastchess"
BOOK="$BENCH_DIR/UHO_Lichess_4852_v1.epd"
RESULTS_DIR="$BENCH_DIR/../results"
CONCURRENCY="${CONCURRENCY:-$(nproc)}"

case "$TC" in
  stc) TCSPEC="8+0.08"; HASH=16 ;;
  ltc) TCSPEC="40+0.4"; HASH=64 ;;
  *) echo "tc must be stc or ltc" >&2; exit 1 ;;
esac

SPRT_ARGS=()
case "$SPRT" in
  better) SPRT_ARGS=(-sprt elo0=0 elo1=5 alpha=0.05 beta=0.05) ;;
  nonreg) SPRT_ARGS=(-sprt elo0=-5 elo1=0 alpha=0.05 beta=0.05) ;;
  none)   SPRT_ARGS=() ;;
  *) echo "sprt must be better, nonreg, or none" >&2; exit 1 ;;
esac

N1=$(basename "${E1%% *}")
N2=$(basename "${E2%% *}")
TAG="${N1}_vs_${N2}_${TC}_$(date +%Y%m%d_%H%M%S)"

mkdir -p "$RESULTS_DIR"
OUT="$RESULTS_DIR/${TAG}.log"

"$FASTCHESS" \
  -engine cmd="$E1" name="$N1" \
  -engine cmd="$E2" name="$N2" \
  -each tc="$TCSPEC" option.Hash=$HASH option.Threads=1 option.Ponder=false \
  -openings file="$BOOK" format=epd order=random \
  -rounds "$ROUNDS" -games 2 \
  -concurrency "$CONCURRENCY" \
  "${SPRT_ARGS[@]}" \
  -pgnout file="$RESULTS_DIR/${TAG}.pgn" \
  -ratinginterval 10 \
  2>&1 | tee "$OUT"

ELO_LINE=$(grep -E '^Elo:' "$OUT" | tail -1 || true)
LOS_LINE=$(grep -E '^LOS:' "$OUT" | tail -1 || true)
GAMES_LINE=$(grep -E '^Games:' "$OUT" | tail -1 || true)

ELO=$(grep -oP 'Elo: \K-?[0-9.]+' <<<"$ELO_LINE" || true)
ELO_ERR=$(grep -oP '\+/- \K[0-9.]+' <<<"$ELO_LINE" | head -1 || true)
LOS=$(grep -oP 'LOS: \K[0-9.]+' <<<"$LOS_LINE" || true)
GAMES=$(grep -oP 'Games: \K[0-9]+' <<<"$GAMES_LINE" || true)

echo "${N1}_vs_${N2},${TC},${GAMES},${ELO},${ELO_ERR},${LOS},,,${SPRT}" >> "$RESULTS_DIR/summary.csv"
echo "Row appended to $RESULTS_DIR/summary.csv"
