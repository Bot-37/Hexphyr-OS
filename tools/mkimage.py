#!/usr/bin/env python3
"""Hexphyr OS release artifact builder."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[1]
BOOTLOADER_MANIFEST = ROOT_DIR / "bootloader" / "Cargo.toml"
KERNEL_MANIFEST = ROOT_DIR / "kernel" / "Cargo.toml"
BOOTLOADER_EFI = (
    ROOT_DIR
    / "bootloader"
    / "target"
    / "x86_64-unknown-uefi"
    / "release"
    / "bootloader.efi"
)
KERNEL_ELF = (
    ROOT_DIR
    / "kernel"
    / "target"
    / "x86_64-unknown-none"
    / "release"
    / "kernel"
)
ROOTFS_DIR = ROOT_DIR / "rootfs"

DEFAULT_OUTPUT_DIR = ROOT_DIR / "dist" / "release"
EFI_IMAGE_NAME = "hexphyr-x86_64-efi.img"
ISO_IMAGE_NAME = "hexphyr-x86_64.iso"
EFI_BOOT_IMAGE_NAME = "efiboot.img"
BOOTX64_NAME = "BOOTX64.EFI"
VERSION_NAME = "VERSION.TXT"
MANIFEST_NAME = "manifest.json"
ESP_MANIFEST_NAME = "MANIFEST.JSON"
CHECKSUMS_NAME = "SHA256SUMS"
INITRAMFS_NAME = "INITRAMFS.BIN"
KERNEL_NAME = "KERNEL.ELF"
EFI_IMAGE_SIZE = 64 * 1024 * 1024
EFI_BOOT_IMAGE_SIZE = 4 * 1024 * 1024
GPT_START_SECTOR = 2048
SECTOR_SIZE = 512


class BuildError(RuntimeError):
    pass


def run(command: list[str], *, cwd: Path | None = None) -> None:
    print("+", " ".join(command))
    subprocess.run(command, cwd=cwd, check=True)


def capture(command: list[str], *, cwd: Path | None = None) -> str:
    return subprocess.check_output(command, cwd=cwd, text=True).strip()


def require_tools(tools: list[str]) -> None:
    missing = [tool for tool in tools if shutil.which(tool) is None]
    if missing:
        raise BuildError(f"missing required tools: {', '.join(missing)}")


def hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_text(path: Path, contents: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(contents, encoding="utf-8")


def copy_tree(source: Path, destination: Path) -> None:
    if destination.exists():
        shutil.rmtree(destination)
    shutil.copytree(source, destination)


def relative_entries(root: Path) -> list[str]:
    entries = ["."]
    paths = sorted(
        root.rglob("*"),
        key=lambda entry: (len(entry.relative_to(root).parts), entry.relative_to(root).as_posix()),
    )
    entries.extend(path.relative_to(root).as_posix() for path in paths)
    return entries


def create_initramfs(rootfs_dir: Path, output_path: Path) -> None:
    entries = relative_entries(rootfs_dir)
    payload = "\n".join(entries) + "\n"
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("wb") as archive:
        subprocess.run(
            ["cpio", "--quiet", "-o", "-H", "newc"],
            cwd=rootfs_dir,
            check=True,
            input=payload,
            text=True,
            stdout=archive,
        )


def create_empty_fat_image(
    image_path: Path,
    size_bytes: int,
    label: str,
    *,
    fat32: bool,
) -> None:
    image_path.parent.mkdir(parents=True, exist_ok=True)
    with image_path.open("wb") as handle:
        handle.truncate(size_bytes)
    command = ["mformat", "-i", str(image_path)]
    if fat32:
        command.append("-F")
    command.extend(["-v", label, "::"])
    run(command)


def populate_fat_image(
    image_path: Path,
    source_dir: Path,
    *,
    offset_bytes: int | None = None,
) -> None:
    image_spec = str(image_path)
    if offset_bytes is not None:
        image_spec = f"{image_spec}@@{offset_bytes}"

    directories = sorted(
        [path for path in source_dir.rglob("*") if path.is_dir()],
        key=lambda path: (len(path.relative_to(source_dir).parts), path.relative_to(source_dir).as_posix()),
    )
    files = sorted(
        [path for path in source_dir.rglob("*") if path.is_file()],
        key=lambda path: path.relative_to(source_dir).as_posix(),
    )

    for directory in directories:
        relative = directory.relative_to(source_dir).as_posix()
        run(["mmd", "-i", image_spec, f"::/{relative}"])

    for file_path in files:
        relative = file_path.relative_to(source_dir).as_posix()
        run(["mcopy", "-i", image_spec, str(file_path), f"::/{relative}"])


def create_gpt_efi_image(image_path: Path, esp_dir: Path) -> None:
    image_path.parent.mkdir(parents=True, exist_ok=True)
    with image_path.open("wb") as handle:
        handle.truncate(EFI_IMAGE_SIZE)

    run(
        [
            "sgdisk",
            "--clear",
            f"--new=1:{GPT_START_SECTOR}:0",
            "--typecode=1:ef00",
            "--change-name=1:Hexphyr EFI",
            str(image_path),
        ]
    )

    offset_bytes = GPT_START_SECTOR * SECTOR_SIZE
    run(["mformat", "-i", f"{image_path}@@{offset_bytes}", "-F", "-v", "HEXPHYR", "::"])
    populate_fat_image(image_path, esp_dir, offset_bytes=offset_bytes)


def create_efi_iso(iso_root: Path, boot_image_path: Path, output_iso: Path) -> None:
    output_iso.parent.mkdir(parents=True, exist_ok=True)
    run(
        [
            "xorriso",
            "-as",
            "mkisofs",
            "-R",
            "-J",
            "-V",
            "HEXPHYR_1_0",
            "-eltorito-alt-boot",
            "-eltorito-platform",
            "efi",
            "-e",
            boot_image_path.name,
            "-no-emul-boot",
            "-efi-boot-part",
            "--efi-boot-image",
            "-isohybrid-gpt-basdat",
            "-o",
            str(output_iso),
            str(iso_root),
        ]
    )


def maybe_git_revision() -> str:
    try:
        return capture(["git", "rev-parse", "--short=12", "HEAD"], cwd=ROOT_DIR)
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def build_metadata(version: str) -> dict[str, object]:
    return {
        "product": "Hexphyr OS",
        "version": version,
        "git_revision": maybe_git_revision(),
        "built_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "boot_mode": "uefi",
        "architecture": "x86_64",
    }


def build_binaries() -> None:
    run(["cargo", "build", "--release"], cwd=BOOTLOADER_MANIFEST.parent)
    run(["cargo", "build", "--release"], cwd=KERNEL_MANIFEST.parent)


def copy_public_cert(cert_path: Path | None, destination_dir: Path) -> str | None:
    if cert_path is None or not cert_path.exists():
        return None

    destination_dir.mkdir(parents=True, exist_ok=True)
    destination = destination_dir / cert_path.name
    shutil.copy2(cert_path, destination)
    return destination.name


def sign_bootloader(bootloader_path: Path, secureboot_dir: Path) -> tuple[Path, dict[str, object]]:
    key_env = os.environ.get("HEXPHYR_SB_KEY")
    cert_env = os.environ.get("HEXPHYR_SB_CERT")
    require_signing = os.environ.get("HEXPHYR_REQUIRE_SIGNING") == "1"
    output_path = secureboot_dir / BOOTX64_NAME
    certificate_name = copy_public_cert(
        Path(cert_env).expanduser().resolve() if cert_env else None,
        secureboot_dir,
    )

    metadata = {
        "signed": False,
        "certificate": certificate_name,
        "reason": "build produced an unsigned EFI binary",
    }

    if key_env and cert_env:
        if shutil.which("sbsign") is None:
            if require_signing:
                raise BuildError(
                    "HEXPHYR_REQUIRE_SIGNING=1 but sbsign is not installed"
                )
        else:
            key_path = Path(key_env).expanduser().resolve()
            cert_path = Path(cert_env).expanduser().resolve()
            if not key_path.exists() or not cert_path.exists():
                raise BuildError("HEXPHYR_SB_KEY or HEXPHYR_SB_CERT does not exist")

            secureboot_dir.mkdir(parents=True, exist_ok=True)
            run(
                [
                    "sbsign",
                    "--key",
                    str(key_path),
                    "--cert",
                    str(cert_path),
                    "--output",
                    str(output_path),
                    str(bootloader_path),
                ]
            )
            metadata = {
                "signed": True,
                "certificate": certificate_name or cert_path.name,
                "reason": "signed with the provided Secure Boot certificate",
            }
            return output_path, metadata

    secureboot_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(bootloader_path, output_path)
    return output_path, metadata


def stage_rootfs(staging_rootfs: Path, metadata: dict[str, object]) -> None:
    if not ROOTFS_DIR.exists():
        raise BuildError(f"rootfs directory is missing: {ROOTFS_DIR}")

    copy_tree(ROOTFS_DIR, staging_rootfs)
    write_text(
        staging_rootfs / "etc" / "hexphyr-release.json",
        json.dumps(metadata, indent=2, sort_keys=True) + "\n",
    )
    write_text(
        staging_rootfs / "etc" / "hexphyr-release.txt",
        "\n".join(
            [
                f"product={metadata['product']}",
                f"version={metadata['version']}",
                f"git_revision={metadata['git_revision']}",
                f"built_at_utc={metadata['built_at_utc']}",
                "boot_mode=uefi",
            ]
        )
        + "\n",
    )


def stage_esp(
    esp_dir: Path,
    *,
    bootx64_path: Path,
    kernel_path: Path,
    initramfs_path: Path,
    metadata: dict[str, object],
) -> None:
    if esp_dir.exists():
        shutil.rmtree(esp_dir)

    (esp_dir / "EFI" / "BOOT").mkdir(parents=True, exist_ok=True)
    (esp_dir / "EFI" / "HEXPHYR").mkdir(parents=True, exist_ok=True)

    shutil.copy2(bootx64_path, esp_dir / "EFI" / "BOOT" / BOOTX64_NAME)
    shutil.copy2(kernel_path, esp_dir / "EFI" / "HEXPHYR" / KERNEL_NAME)
    shutil.copy2(initramfs_path, esp_dir / "EFI" / "HEXPHYR" / INITRAMFS_NAME)
    write_text(
        esp_dir / "EFI" / "HEXPHYR" / VERSION_NAME,
        "\n".join(
            [
                f"Hexphyr OS {metadata['version']}",
                f"git_revision={metadata['git_revision']}",
                f"built_at_utc={metadata['built_at_utc']}",
            ]
        )
        + "\n",
    )
    write_text(
        esp_dir / "EFI" / "HEXPHYR" / ESP_MANIFEST_NAME,
        json.dumps(metadata, indent=2, sort_keys=True) + "\n",
    )


def write_manifest(
    manifest_path: Path,
    metadata: dict[str, object],
    secureboot: dict[str, object],
    artifacts: dict[str, dict[str, object]],
) -> None:
    payload = {
        **metadata,
        "release_contract": {
            "bootloader_to_kernel_abi": "bootabi::BootInfo",
            "release_boot_path": "UEFI",
            "artifacts": {
                "efi_image": EFI_IMAGE_NAME,
                "iso_image": ISO_IMAGE_NAME,
                "boot_binary": BOOTX64_NAME,
                "initramfs": INITRAMFS_NAME,
            },
        },
        "secure_boot": secureboot,
        "artifacts": artifacts,
    }
    write_text(manifest_path, json.dumps(payload, indent=2, sort_keys=True) + "\n")


def write_checksums(checksums_path: Path, files: list[Path]) -> None:
    lines = [
        f"{hash_file(path)}  {path.name}"
        for path in sorted(files, key=lambda item: item.name)
    ]
    write_text(checksums_path, "\n".join(lines) + "\n")


def build_release(output_dir: Path, *, version: str, skip_build: bool) -> None:
    require_tools(["cargo", "cpio", "mcopy", "mformat", "mmd", "sgdisk", "xorriso"])

    if not skip_build:
        build_binaries()

    if not BOOTLOADER_EFI.exists():
        raise BuildError(f"bootloader artifact not found: {BOOTLOADER_EFI}")
    if not KERNEL_ELF.exists():
        raise BuildError(f"kernel artifact not found: {KERNEL_ELF}")

    metadata = build_metadata(version)
    staging_dir = output_dir / "staging"
    rootfs_staging = staging_dir / "rootfs"
    esp_dir = staging_dir / "esp"
    iso_root = staging_dir / "iso"
    secureboot_dir = output_dir / "secureboot"
    artifacts_dir = output_dir / "artifacts"
    initramfs_path = staging_dir / INITRAMFS_NAME
    efi_boot_image = staging_dir / EFI_BOOT_IMAGE_NAME
    efi_disk_image = artifacts_dir / EFI_IMAGE_NAME
    iso_image = artifacts_dir / ISO_IMAGE_NAME
    manifest_path = artifacts_dir / MANIFEST_NAME
    checksums_path = artifacts_dir / CHECKSUMS_NAME
    boot_binary_path = artifacts_dir / BOOTX64_NAME

    if output_dir.exists():
        shutil.rmtree(output_dir)
    artifacts_dir.mkdir(parents=True, exist_ok=True)

    stage_rootfs(rootfs_staging, metadata)
    create_initramfs(rootfs_staging, initramfs_path)

    signed_bootloader_path, secureboot = sign_bootloader(BOOTLOADER_EFI, secureboot_dir)
    stage_esp(
        esp_dir,
        bootx64_path=signed_bootloader_path,
        kernel_path=KERNEL_ELF,
        initramfs_path=initramfs_path,
        metadata=metadata,
    )

    shutil.copy2(esp_dir / "EFI" / "BOOT" / BOOTX64_NAME, boot_binary_path)

    copy_tree(esp_dir / "EFI", iso_root / "EFI")
    shutil.copy2(initramfs_path, artifacts_dir / INITRAMFS_NAME)
    shutil.copy2(KERNEL_ELF, artifacts_dir / KERNEL_NAME)
    shutil.copy2(esp_dir / "EFI" / "HEXPHYR" / VERSION_NAME, artifacts_dir / VERSION_NAME)

    create_empty_fat_image(efi_boot_image, EFI_BOOT_IMAGE_SIZE, "HEXPHYR", fat32=False)
    populate_fat_image(efi_boot_image, esp_dir)
    shutil.copy2(efi_boot_image, iso_root / EFI_BOOT_IMAGE_NAME)

    create_gpt_efi_image(efi_disk_image, esp_dir)
    create_efi_iso(iso_root, efi_boot_image, iso_image)

    artifact_paths = [
        boot_binary_path,
        artifacts_dir / INITRAMFS_NAME,
        artifacts_dir / KERNEL_NAME,
        artifacts_dir / VERSION_NAME,
        efi_disk_image,
        iso_image,
    ]
    artifact_metadata = {
        path.name: {
            "size_bytes": path.stat().st_size,
            "sha256": hash_file(path),
        }
        for path in artifact_paths
    }

    write_manifest(manifest_path, metadata, secureboot, artifact_metadata)
    write_checksums(checksums_path, artifact_paths + [manifest_path])

    print(f"release artifacts written to {artifacts_dir}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build Hexphyr release artifacts")
    parser.add_argument(
        "--output-dir",
        default=str(DEFAULT_OUTPUT_DIR),
        help="Output directory for staged files and release artifacts",
    )
    parser.add_argument(
        "--version",
        default=os.environ.get("HEXPHYR_VERSION", "1.0.0"),
        help="Release version string",
    )
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="Reuse existing release binaries instead of rebuilding them",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        build_release(
            Path(args.output_dir).expanduser().resolve(),
            version=args.version,
            skip_build=args.skip_build,
        )
    except BuildError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    except subprocess.CalledProcessError as error:
        print(f"error: command failed with exit code {error.returncode}", file=sys.stderr)
        return error.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
