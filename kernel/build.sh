#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "kernel/build.sh now delegates to the UEFI release pipeline."
exec python3 "${ROOT_DIR}/tools/mkimage.py" "$@"
