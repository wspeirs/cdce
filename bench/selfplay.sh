#!/usr/bin/env bash
# Self-play harness: run fastchess with the same engine binary on both sides
# with custom UCI options per side (e.g. SetEval or Val_* terms).
#
# Usage: bench/selfplay.sh <name-A> <opt-A...> -- <name-B> <opt-B...> [games]
# Example: bench/selfplay.sh A Val_Knight=320 -- B Val_Knight=420 20
set -euo pipefail

usage() {
  echo "Usage: bench/selfplay.sh <name-A> <opt-A...> -- <name-B> <opt-B...> [games]"
  echo "Example: bench/selfplay.sh A Val_Knight=320 -- B Val_Knight=420 20"
}

# Check for help flags
for arg in "$@"; do
  if [[ "$arg" == "-h" || "$arg" == "--help" ]]; then
    usage
    exit 0
  fi
done

if [[ $# -eq 0 ]]; then
  usage
  exit 1
fi

args=("$@")

# Find the '--' separator between side A and side B
found_sep=false
sep_idx=-1
for i in "${!args[@]}"; do
  if [[ "${args[i]}" == "--" ]]; then
    found_sep=true
    sep_idx=$i
    break
  fi
done

if ! $found_sep; then
  echo "Error: missing '--' separator between side A and side B" >&2
  usage >&2
  exit 1
fi

if [[ $sep_idx -eq 0 ]]; then
  echo "Error: missing engine name for side A (<name-A>)" >&2
  usage >&2
  exit 1
fi

if [[ $((sep_idx + 1)) -ge ${#args[@]} ]]; then
  echo "Error: missing engine name for side B (<name-B>)" >&2
  usage >&2
  exit 1
fi

NAME_A="${args[0]}"
OPTS_A=()
if [[ $sep_idx -gt 1 ]]; then
  OPTS_A=("${args[@]:1:sep_idx-1}")
fi

NAME_B="${args[sep_idx + 1]}"
REMAINING_B=()
b_start=$((sep_idx + 2))
if [[ $b_start -lt ${#args[@]} ]]; then
  REMAINING_B=("${args[@]:b_start}")
fi

GAMES=20
OPTS_B=()
if [[ ${#REMAINING_B[@]} -gt 0 ]]; then
  last_idx=$((${#REMAINING_B[@]} - 1))
  last_token="${REMAINING_B[last_idx]}"
  if [[ "$last_token" =~ ^[0-9]+$ ]]; then
    GAMES="$last_token"
    if [[ $last_idx -gt 0 ]]; then
      OPTS_B=("${REMAINING_B[@]:0:last_idx}")
    fi
  else
    OPTS_B=("${REMAINING_B[@]}")
  fi
fi

if ! [[ "$GAMES" =~ ^[0-9]+$ ]] || (( GAMES < 2 )); then
  echo "Error: games must be a positive integer >= 2 (got '$GAMES')" >&2
  exit 1
fi

ROUNDS=$(( GAMES / 2 ))

# Option ordering note:
# Fastchess preserves the command-line argument order when sending UCI `setoption` commands
# to engines. The UCI engine resets evaluation parameters to defaults when `SetEval` is set.
# Therefore, `SetEval` must always be sent BEFORE any `Val_*` option overrides.
# If a side does not specify `SetEval`, we default to `option.SetEval=CDCE` first.
# If a side specifies its own `SetEval`, we ensure it is emitted first before other options.
build_engine_opts() {
  local seteval_opt="option.SetEval=CDCE"
  local other_opts=()

  for opt in "$@"; do
    local clean_opt="${opt#option.}"
    if [[ "$clean_opt" != *"="* ]]; then
      echo "Error: option '$opt' must be in Name=Value format" >&2
      exit 1
    fi
    if [[ "$clean_opt" =~ ^SetEval= ]]; then
      seteval_opt="option.$clean_opt"
    else
      other_opts+=("option.$clean_opt")
    fi
  done

  ENGINE_OPTS=("$seteval_opt" "${other_opts[@]}")
}

build_engine_opts "${OPTS_A[@]}"
ENGINE_A_OPTS=("${ENGINE_OPTS[@]}")

build_engine_opts "${OPTS_B[@]}"
ENGINE_B_OPTS=("${ENGINE_OPTS[@]}")

# Environment and path setup
BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$BENCH_DIR/.." && pwd)"
FASTCHESS="$BENCH_DIR/bin/fastchess"
BOOK="$BENCH_DIR/UHO_Lichess_4852_v1.epd"
RESULTS_DIR="$REPO_ROOT/results"
CDCE_BIN="${CDCE_BIN:-$REPO_ROOT/target/release/cdce}"
CONCURRENCY="${CONCURRENCY:-$(nproc)}"

# Validation
if [[ ! -x "$CDCE_BIN" ]]; then
  echo "Error: CDCE binary is not executable or does not exist: $CDCE_BIN" >&2
  exit 1
fi

if [[ ! -x "$FASTCHESS" ]]; then
  echo "Error: fastchess binary not found or not executable: $FASTCHESS" >&2
  exit 1
fi

if [[ ! -f "$BOOK" ]]; then
  echo "Error: opening book not found: $BOOK" >&2
  exit 1
fi

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TAG="selfplay_${NAME_A}_vs_${NAME_B}_${TIMESTAMP}"
mkdir -p "$RESULTS_DIR"
PGN="$RESULTS_DIR/${TAG}.pgn"
LOG="$RESULTS_DIR/${TAG}.log"

# Run fastchess match
"$FASTCHESS" \
  -engine cmd="$CDCE_BIN" name="$NAME_A" "${ENGINE_A_OPTS[@]}" \
  -engine cmd="$CDCE_BIN" name="$NAME_B" "${ENGINE_B_OPTS[@]}" \
  -each st=0.1 option.Hash=16 option.Threads=1 option.Ponder=false \
  -openings file="$BOOK" format=epd order=random \
  -rounds "$ROUNDS" -games 2 \
  -concurrency "$CONCURRENCY" \
  -pgnout file="$PGN" \
  -ratinginterval 10 \
  2>&1 | tee "$LOG"

# Extract summary from log
ELO_LINE=$(grep -E '^Elo:' "$LOG" | tail -1 || true)
GAMES_LINE=$(grep -E '^Games:' "$LOG" | tail -1 || true)

ELO=$(grep -oP '^Elo: \K-?[0-9.]+' <<<"$ELO_LINE" | head -1 || true)
ELO_ERR=$(grep -oP '\+/- \K[0-9.]+' <<<"$ELO_LINE" | head -1 || true)
GAMES_PLAYED=$(grep -oP 'Games: \K[0-9]+' <<<"$GAMES_LINE" | head -1 || true)
WINS=$(grep -oP 'Wins: \K[0-9]+' <<<"$GAMES_LINE" | head -1 || true)
LOSSES=$(grep -oP 'Losses: \K[0-9]+' <<<"$GAMES_LINE" | head -1 || true)
DRAWS=$(grep -oP 'Draws: \K[0-9]+' <<<"$GAMES_LINE" | head -1 || true)
POINTS=$(grep -oP 'Points: \K[0-9.]+' <<<"$GAMES_LINE" | head -1 || true)

if [[ -n "$ELO" && -n "$GAMES_PLAYED" ]]; then
  echo "Score: +${WINS:-0} -${LOSSES:-0} =${DRAWS:-0} (${POINTS:-0}/${GAMES_PLAYED}), Elo: ${ELO} +/- ${ELO_ERR}"
else
  echo "Match completed. See log: $LOG"
fi
