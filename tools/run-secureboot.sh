#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="${ROOT_DIR}/dist/release/artifacts/manifest.json"

if [ ! -f "${MANIFEST}" ] || ! grep -q '"signed": true' "${MANIFEST}"; then
  echo "Secure Boot artifact is not signed. Rebuild with HEXPHYR_SB_KEY and HEXPHYR_SB_CERT plus sbsign installed." >&2
  exit 1
fi

export HEXPHYR_SECUREBOOT=1
exec "${ROOT_DIR}/tools/run-ovmf.sh"
