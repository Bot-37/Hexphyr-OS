# Hexphyr OS

Hexphyr OS 1.0 ships as a single `x86_64` UEFI product with a native Rust
bootloader, a 64-bit kernel, a framebuffer desktop, and a packaged initramfs
contract. The supported release artifacts are:

- `dist/release/artifacts/hexphyr-x86_64.iso`
- `dist/release/artifacts/hexphyr-x86_64-efi.img`
- `dist/release/artifacts/BOOTX64.EFI`
- `dist/release/artifacts/manifest.json`
- `dist/release/artifacts/SHA256SUMS`

## Release flow

The release boot path is UEFI only. The bootloader loads:

- `EFI/BOOT/BOOTX64.EFI`
- `EFI/HEXPHYR/KERNEL.ELF`
- `EFI/HEXPHYR/INITRAMFS.BIN`
- `EFI/HEXPHYR/VERSION.TXT`
- `EFI/HEXPHYR/MANIFEST.JSON`

The kernel consumes the shared `bootabi::BootInfo` contract and validates the
initramfs layout at boot. The legacy Multiboot path remains in the kernel only
as a developer fallback and is not part of the supported release workflow.

## Prerequisites

Install the Rust targets pinned by `rust-toolchain.toml` and the host tools used
for packaging and simulation:

```bash
rustup target add x86_64-unknown-none x86_64-unknown-uefi --toolchain nightly
sudo apt-get install \
  cpio \
  dosfstools \
  gdisk \
  mtools \
  ovmf \
  python3 \
  qemu-system-x86 \
  xorriso
```

Optional Secure Boot tooling:

```bash
sudo apt-get install efitools openssl sbsigntool uuid-runtime
```

## Build

Build the release bundle:

```bash
make release-artifacts
```

Boot the EFI disk image:

```bash
make run-ovmf
```

Boot the ISO instead of the disk image:

```bash
make run-iso
```

Run a headless smoke test:

```bash
HEXPHYR_HEADLESS=1 HEXPHYR_TIMEOUT_SEC=20 make run-ovmf-headless
```

## Docker

Build the containerized release environment:

```bash
make docker-build
```

Build the release bundle and run the QEMU smoke test in Docker:

```bash
make docker-sim
```

## Secure Boot

Generate an owner certificate bundle:

```bash
./tools/secureboot/generate-owner-keys.sh dist/secureboot-owner
```

Build signed artifacts:

```bash
HEXPHYR_SB_KEY=dist/secureboot-owner/hexphyr-owner.key \
HEXPHYR_SB_CERT=dist/secureboot-owner/hexphyr-owner.crt \
make release-artifacts
```

If your enrolled OVMF variables live in a custom file, pass it to the runner:

```bash
HEXPHYR_OVMF_VARS_TEMPLATE=/path/to/OVMF_VARS.fd make run-secureboot
```

Additional Secure Boot notes live in [tools/secureboot/README.md](/home/shadow37/Projects/OS%20building/hexphyr-os/tools/secureboot/README.md).

## Current release scope

The release pipeline, boot ABI, UEFI loader, framebuffer UI, initramfs packaging,
artifact manifest, checksum generation, Docker environment, and CI smoke test are
implemented and verified in QEMU/OVMF.

The initramfs already carries the expected `/sbin/init`, `/bin/sh`, and command
layout so the kernel can validate the contract on boot. User-mode execution,
syscalls, and keyboard-driven shell bring-up are staged next; they are not yet
active in the current kernel runtime.
