#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "kernel/run.sh now delegates to the UEFI release runner."
exec "${ROOT_DIR}/tools/run-ovmf.sh" "$@"
