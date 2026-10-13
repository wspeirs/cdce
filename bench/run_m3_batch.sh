#!/usr/bin/env bash
# Sequential M3 batch on the current build. Each match uses all cores, so runs
# are strictly one after another. Every record gets a fresh TAG; existing
# records are never overwritten (the scripts refuse).
#
#  1. Core and Material reruns vs MateOnCE on the standard value set (400 games each)
#  2. Seven one-family ablations vs full standard CDCE (400 games each)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

STD="option.Val_Pawn=100 option.Val_Knight=320 option.Val_Bishop=330 option.Val_Rook=500 option.Val_Queen=900"

TAG=m5_std_core_cur_vs_mateonce ROUNDS=200 CDCE_EVAL=CDCE-Core CDCE_OPTS="$STD" \
  bash bench/smoke_m2_fastchess.sh
TAG=m5_std_material_cur_vs_mateonce ROUNDS=200 CDCE_EVAL=CDCE-Material CDCE_OPTS="$STD" \
  bash bench/smoke_m2_fastchess.sh

for v in CDCE-NoOneWay CDCE-NoForks CDCE-NoRook CDCE-NoMobility CDCE-NoKingSafety CDCE-NoPawnStructure CDCE-OneWayDefended; do
  bash bench/ablation_match.sh "$v" 200
done

echo "M3 batch finished."
