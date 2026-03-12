.PHONY: all bootloader kernel release release-artifacts run-ovmf run-iso run-secureboot run-ovmf-headless clean help docker-build docker-sim

PYTHON ?= python3
IMAGE_NAME ?= hexphyr-os:dev

all: release

bootloader:
	cd bootloader && cargo build --release

kernel:
	cd kernel && cargo build --release

release: release-artifacts

release-artifacts:
	$(PYTHON) tools/mkimage.py

run-ovmf: release-artifacts
	./tools/run-ovmf.sh

run-iso: release-artifacts
	HEXPHYR_BOOT_MODE=iso ./tools/run-ovmf.sh

run-secureboot: release-artifacts
	./tools/run-secureboot.sh

run-ovmf-headless: release-artifacts
	HEXPHYR_HEADLESS=1 HEXPHYR_TIMEOUT_SEC=$${HEXPHYR_TIMEOUT_SEC:-20} ./tools/run-ovmf.sh

clean:
	cargo clean --manifest-path bootloader/Cargo.toml
	cargo clean --manifest-path kernel/Cargo.toml
	rm -rf dist

docker-build:
	docker build -t $(IMAGE_NAME) .

docker-sim:
	./tools/docker-sim.sh

help:
	@echo "Hexphyr OS release targets:"
	@echo "  release             Build the UEFI release artifacts"
	@echo "  release-artifacts   Build the UEFI ISO, EFI image, and manifest bundle"
	@echo "  run-ovmf            Boot the EFI disk image in QEMU/OVMF"
	@echo "  run-iso             Boot the UEFI ISO in QEMU/OVMF"
	@echo "  run-secureboot      Boot the signed EFI image with Secure Boot firmware"
	@echo "  run-ovmf-headless   Headless QEMU/OVMF smoke test"
	@echo "  bootloader          Build the release UEFI bootloader"
	@echo "  kernel              Build the release kernel"
	@echo "  docker-build        Build the containerized release environment"
	@echo "  docker-sim          Build the container and run a headless QEMU smoke test"
	@echo "  clean               Remove cargo outputs and dist artifacts"
