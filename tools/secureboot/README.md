# Secure Boot

Hexphyr signs the UEFI bootloader with `sbsign` when these environment variables
are present during `make release-artifacts` or `python3 tools/mkimage.py`:

- `HEXPHYR_SB_KEY`
- `HEXPHYR_SB_CERT`

Generate an owner keypair:

```bash
./tools/secureboot/generate-owner-keys.sh dist/secureboot-owner
```

This produces:

- `hexphyr-owner.key`
- `hexphyr-owner.crt`
- `hexphyr-owner.cer`
- `hexphyr-owner.esl` when `efitools` is installed

Sign a bootloader manually:

```bash
./tools/secureboot/sign-efi.sh \
  bootloader/target/x86_64-unknown-uefi/release/bootloader.efi \
  dist/signed-BOOTX64.EFI \
  dist/secureboot-owner/hexphyr-owner.key \
  dist/secureboot-owner/hexphyr-owner.crt
```

Owner-enrolled Secure Boot flow:

1. Generate the owner cert bundle.
2. Enroll `hexphyr-owner.cer` or `hexphyr-owner.esl` with your firmware UI or `KeyTool.efi`.
3. Rebuild release artifacts with `HEXPHYR_SB_KEY` and `HEXPHYR_SB_CERT` set.
4. Boot the signed image with `make run-secureboot`.

If you already have an enrolled OVMF variable store, point the runner at it with
`HEXPHYR_OVMF_VARS_TEMPLATE=/path/to/OVMF_VARS.fd`.
