#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_NAME="${IMAGE_NAME:-hexphyr-os:dev}"
SIM_TIMEOUT_SEC="${SIM_TIMEOUT_SEC:-25}"

docker build -t "${IMAGE_NAME}" "${ROOT_DIR}"

docker run --rm -it \
  -e SIM_TIMEOUT_SEC="${SIM_TIMEOUT_SEC}" \
  -e HEXPHYR_HEADLESS=1 \
  "${IMAGE_NAME}" \
  /workspace/tools/simulate-kernel.sh
