#!/usr/bin/env bash
set -euo pipefail

ISO=hexphyr.iso
if [ ! -f "${ISO}" ]; then
  echo "ISO ${ISO} not found. Run ./build.sh first."
  exit 1
fi

qemu-system-x86_64 \
  -m 512M \
  -serial mon:stdio \
  -cdrom "${ISO}" \
  -boot d \
  -no-reboot \
  -no-shutdown \
  -d guest_errors \
  -enable-kvm
