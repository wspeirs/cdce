#!/usr/bin/env bash
# Long time control (40+0.4) SPRT run. See run_match.sh for details.
# Usage: run_ltc.sh <engine1_cmd> <engine2_cmd> [rounds] [sprt=better|nonreg|none]
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$DIR/run_match.sh" ltc "$@"
