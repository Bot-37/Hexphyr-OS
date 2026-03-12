#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-dist/release/secureboot/owner}"
CN="${HEXPHYR_SB_CN:-Hexphyr Owner}"
DAYS="${HEXPHYR_SB_DAYS:-3650}"

mkdir -p "${OUT_DIR}"

KEY_PATH="${OUT_DIR}/hexphyr-owner.key"
CRT_PATH="${OUT_DIR}/hexphyr-owner.crt"
CER_PATH="${OUT_DIR}/hexphyr-owner.cer"

openssl req \
  -new \
  -x509 \
  -newkey rsa:2048 \
  -sha256 \
  -nodes \
  -days "${DAYS}" \
  -subj "/CN=${CN}/" \
  -keyout "${KEY_PATH}" \
  -out "${CRT_PATH}"

chmod 600 "${KEY_PATH}"
openssl x509 -outform DER -in "${CRT_PATH}" -out "${CER_PATH}"

if command -v cert-to-efi-sig-list >/dev/null 2>&1; then
  GUID="$(
    if command -v uuidgen >/dev/null 2>&1; then
      uuidgen
    else
      cat /proc/sys/kernel/random/uuid
    fi
  )"
  cert-to-efi-sig-list -g "${GUID}" "${CRT_PATH}" "${OUT_DIR}/hexphyr-owner.esl"
fi

printf 'Generated Secure Boot owner keys in %s\n' "${OUT_DIR}"
