#!/usr/bin/env bash
set -euo pipefail

# 1) Build the kernel (debug; change to --release for release)
cargo build

# 2) prepare iso dir
ISO_DIR=iso
BOOT_DIR=${ISO_DIR}/boot
GRUB_DIR=${BOOT_DIR}/grub

rm -rf "${ISO_DIR}"
mkdir -p "${GRUB_DIR}"

# 3) Copy kernel ELF
# Adjust this path if your target triple / crate name differs.
KERNEL_ELF=target/x86_64-unknown-none/debug/kernel
if [ ! -f "${KERNEL_ELF}" ]; then
  echo "Error: kernel ELF not found at ${KERNEL_ELF}"
  exit 1
fi

cp "${KERNEL_ELF}" "${BOOT_DIR}/kernel.elf"

# 4) Copy grub.cfg
cat > "${GRUB_DIR}/grub.cfg" <<'EOF'
set timeout=0
set default=0
set gfxmode=1024x768x32
set gfxpayload=keep

menuentry "Hexphyr OS" {
  multiboot2 /boot/kernel.elf
  boot
}
EOF

# 5) Create iso (grub-mkrescue)
# On many distros you need grub and xorriso installed (package name: grub2, grub-pc-bin, grub-mkrescue, xorriso)
grub-mkrescue -o hexphyr.iso "${ISO_DIR}" || {
  echo "grub-mkrescue failed: ensure grub and xorriso are installed"
  exit 1
}

echo "hexphyr.iso created"
