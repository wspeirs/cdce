#!/usr/bin/env bash
# Short time control (8+0.08) SPRT run. See run_match.sh for details.
# Usage: run_stc.sh <engine1_cmd> <engine2_cmd> [rounds] [sprt=better|nonreg|none]
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$DIR/run_match.sh" stc "$@"
