# Hexphyr OS Build System

.PHONY: all bootloader kernel image clean run qemu

# Toolchain
RUSTC = rustc
CARGO = cargo
OBJCOPY = rust-objcopy
QEMU = qemu-system-x86_64

# Directories
BOOTLOADER_DIR = bootloader
KERNEL_DIR = kernel
ISO_DIR = iso
TOOLS_DIR = tools

# Targets
all: bootloader kernel image

bootloader:
	cd $(BOOTLOADER_DIR) && $(CARGO) build --release

kernel:
	cd $(KERNEL_DIR) && $(CARGO) build --release
	$(OBJCOPY) -O binary \
		$(KERNEL_DIR)/target/x86_64-unknown-none/release/kernel \
		$(KERNEL_DIR)/kernel.bin

image: bootloader kernel
	mkdir -p $(ISO_DIR)/EFI/BOOT
	cp $(BOOTLOADER_DIR)/target/x86_64-unknown-uefi/release/bootloader.efi \
		$(ISO_DIR)/EFI/BOOT/BOOTX64.EFI
	cp $(KERNEL_DIR)/kernel.bin $(ISO_DIR)/kernel.elf
	grub-mkrescue -o hexphyr.iso $(ISO_DIR)

clean:
	cd $(BOOTLOADER_DIR) && $(CARGO) clean
	cd $(KERNEL_DIR) && $(CARGO) clean
	rm -rf $(ISO_DIR)
	rm -f hexphyr.iso
	rm -f $(KERNEL_DIR)/kernel.bin

run: image
	$(QEMU) \
		-machine q35 \
		-cpu qemu64 \
		-smp 4 \
		-m 2G \
		-drive if=pflash,format=raw,readonly=on,file=/usr/share/ovmf/OVMF_CODE.fd \
		-drive if=pflash,format=raw,readonly=on,file=/usr/share/ovmf/OVMF_VARS.fd \
		-drive format=raw,file=fat:rw:$(ISO_DIR) \
		-net none \
		-serial stdio \
		-no-reboot \
		-no-shutdown

qemu-debug: image
	$(QEMU) \
		-machine q35 \
		-cpu qemu64 \
		-smp 4 \
		-m 2G \
		-drive if=pflash,format=raw,readonly=on,file=/usr/share/ovmf/OVMF_CODE.fd \
		-drive if=pflash,format=raw,readonly=on,file=/usr/share/ovmf/OVMF_VARS.fd \
		-drive format=raw,file=fat:rw:$(ISO_DIR) \
		-net none \
		-serial stdio \
		-no-reboot \
		-no-shutdown \
		-s -S

help:
	@echo "Hexphyr OS Build Targets:"
	@echo "  all          - Build everything"
	@echo "  bootloader   - Build UEFI bootloader"
	@echo "  kernel       - Build kernel"
	@echo "  image        - Create bootable ISO"
	@echo "  run          - Run in QEMU"
	@echo "  qemu-debug   - Run QEMU with GDB stub"
	@echo "  clean        - Clean all build artifacts"