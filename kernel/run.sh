#!/usr/bin/env bash
set -euo pipefail

ISO="${1:-hexphyr.iso}"
if [ ! -f "${ISO}" ]; then
  echo "ISO ${ISO} not found. Run ./build.sh first."
  exit 1
fi

QEMU_ARGS=(
  -m 512M
  -serial mon:stdio
  -cdrom "${ISO}"
  -boot d
  -no-reboot
  -no-shutdown
  -d guest_errors
)

if [ "${HEXPHYR_HEADLESS:-0}" = "1" ]; then
  QEMU_ARGS+=(-display none)
fi

if [ "${HEXPHYR_KVM:-0}" = "1" ]; then
  QEMU_ARGS+=(-enable-kvm)
fi

qemu-system-x86_64 "${QEMU_ARGS[@]}"
