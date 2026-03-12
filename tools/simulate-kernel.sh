#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KERNEL_DIR="${ROOT_DIR}/kernel"

cd "${KERNEL_DIR}"
./build.sh

# Default to headless for non-interactive simulation.
export HEXPHYR_HEADLESS="${HEXPHYR_HEADLESS:-1}"

if [ -n "${SIM_TIMEOUT_SEC:-}" ]; then
  timeout "${SIM_TIMEOUT_SEC}" ./run.sh || true
else
  ./run.sh
fi
