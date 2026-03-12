#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACT_DIR="${ROOT_DIR}/dist/release/artifacts"
EFI_IMAGE="${ARTIFACT_DIR}/hexphyr-x86_64-efi.img"
ISO_IMAGE="${ARTIFACT_DIR}/hexphyr-x86_64.iso"
RUN_DIR="${ROOT_DIR}/dist/release/run"

find_first() {
  local candidate
  for candidate in "$@"; do
    if [ -f "${candidate}" ]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done
  return 1
}

if [ ! -f "${EFI_IMAGE}" ] || [ ! -f "${ISO_IMAGE}" ]; then
  python3 "${ROOT_DIR}/tools/mkimage.py"
fi

SECUREBOOT="${HEXPHYR_SECUREBOOT:-0}"
BOOT_MODE="${HEXPHYR_BOOT_MODE:-disk}"

if [ "${SECUREBOOT}" = "1" ]; then
  OVMF_CODE="$(find_first \
    /usr/share/edk2/x64/OVMF_CODE.secboot.4m.fd \
    /usr/share/OVMF/OVMF_CODE.secboot.fd \
    /usr/share/OVMF/OVMF_CODE.secboot.4m.fd \
    /usr/share/OVMF/OVMF_CODE_4M.secboot.fd)"
else
  OVMF_CODE="$(find_first \
    /usr/share/edk2/x64/OVMF_CODE.4m.fd \
    /usr/share/edk2/x64/OVMF_CODE.fd \
    /usr/share/OVMF/OVMF_CODE.fd \
    /usr/share/OVMF/OVMF_CODE_4M.fd \
    /usr/share/ovmf/OVMF_CODE.fd)"
fi

OVMF_VARS_TEMPLATE="${HEXPHYR_OVMF_VARS_TEMPLATE:-}"
if [ -z "${OVMF_VARS_TEMPLATE}" ]; then
  OVMF_VARS_TEMPLATE="$(find_first \
    /usr/share/edk2/x64/OVMF_VARS.4m.fd \
    /usr/share/edk2/x64/OVMF_VARS.fd \
    /usr/share/OVMF/OVMF_VARS.fd \
    /usr/share/OVMF/OVMF_VARS_4M.fd \
    /usr/share/ovmf/OVMF_VARS.fd)"
fi

mkdir -p "${RUN_DIR}"
OVMF_VARS="${RUN_DIR}/OVMF_VARS.fd"
cp "${OVMF_VARS_TEMPLATE}" "${OVMF_VARS}"

QEMU_ARGS=(
  -machine q35
  -cpu qemu64
  -smp 2
  -m 1024
  -vga std
  -device qemu-xhci
  -device usb-kbd
  -drive "if=pflash,format=raw,readonly=on,file=${OVMF_CODE}"
  -drive "if=pflash,format=raw,file=${OVMF_VARS}"
  -net none
  -serial mon:stdio
  -no-reboot
  -no-shutdown
)

case "${BOOT_MODE}" in
  disk)
    QEMU_ARGS+=(-drive "format=raw,file=${EFI_IMAGE}")
    ;;
  iso)
    QEMU_ARGS+=(-cdrom "${ISO_IMAGE}" -boot d)
    ;;
  *)
    echo "Unsupported HEXPHYR_BOOT_MODE: ${BOOT_MODE}" >&2
    exit 1
    ;;
esac

if [ "${HEXPHYR_HEADLESS:-0}" = "1" ]; then
  QEMU_ARGS+=(-display none)
fi

if [ "${HEXPHYR_KVM:-0}" = "1" ]; then
  QEMU_ARGS+=(-enable-kvm)
fi

if [ -n "${HEXPHYR_TIMEOUT_SEC:-}" ] && command -v timeout >/dev/null 2>&1; then
  set +e
  timeout "${HEXPHYR_TIMEOUT_SEC}" qemu-system-x86_64 "${QEMU_ARGS[@]}"
  STATUS=$?
  set -e
  if [ "${STATUS}" -eq 124 ] && [ "${HEXPHYR_TIMEOUT_IS_SUCCESS:-1}" = "1" ]; then
    exit 0
  fi
  exit "${STATUS}"
fi

exec qemu-system-x86_64 "${QEMU_ARGS[@]}"
