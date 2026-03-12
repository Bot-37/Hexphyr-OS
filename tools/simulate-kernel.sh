#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python3 "${ROOT_DIR}/tools/mkimage.py"

# Default to headless for non-interactive simulation.
export HEXPHYR_HEADLESS="${HEXPHYR_HEADLESS:-1}"
export HEXPHYR_TIMEOUT_SEC="${SIM_TIMEOUT_SEC:-${HEXPHYR_TIMEOUT_SEC:-25}}"

"${ROOT_DIR}/tools/run-ovmf.sh" || true
