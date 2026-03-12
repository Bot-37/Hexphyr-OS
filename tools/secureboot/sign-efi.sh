#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <input-efi> <output-efi> <signing-key> <signing-cert>" >&2
  exit 1
fi

if ! command -v sbsign >/dev/null 2>&1; then
  echo "sbsign is required to sign EFI binaries" >&2
  exit 1
fi

INPUT_EFI="$1"
OUTPUT_EFI="$2"
SIGNING_KEY="$3"
SIGNING_CERT="$4"

sbsign \
  --key "${SIGNING_KEY}" \
  --cert "${SIGNING_CERT}" \
  --output "${OUTPUT_EFI}" \
  "${INPUT_EFI}"

printf 'Signed EFI binary written to %s\n' "${OUTPUT_EFI}"
