#!/usr/bin/env python3
"""
Hexphyr OS Image Builder
Creates bootable UEFI disk images
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

class ImageBuilder:
    def __init__(self, output="hexphyr.iso"):
        self.output = Path(output)
        self.iso_dir = Path("iso")
        self.bootloader = Path("bootloader/target/x86_64-unknown-uefi/release/bootloader.efi")
        self.kernel = Path("kernel/kernel.bin")
        
    def prepare_directories(self):
        """Create ISO directory structure"""
        self.iso_dir.mkdir(exist_ok=True)
        (self.iso_dir / "EFI/BOOT").mkdir(parents=True, exist_ok=True)
        
    def copy_files(self):
        """Copy bootloader and kernel to ISO directory"""
        if not self.bootloader.exists():
            print(f"Error: Bootloader not found at {self.bootloader}")
            sys.exit(1)
            
        if not self.kernel.exists():
            print(f"Error: Kernel not found at {self.kernel}")
            sys.exit(1)
            
        # Copy bootloader
        shutil.copy(self.bootloader, self.iso_dir / "EFI/BOOT/BOOTX64.EFI")
        
        # Copy kernel (rename to .elf for bootloader)
        shutil.copy(self.kernel, self.iso_dir / "kernel.elf")
        
    def create_image(self):
        """Create bootable ISO using grub-mkrescue"""
        try:
            subprocess.run([
                "grub-mkrescue",
                "-o", str(self.output),
                str(self.iso_dir),
                "--compress=xz"
            ], check=True)
            print(f"Created bootable image: {self.output}")
            print(f"Size: {self.output.stat().st_size / 1024 / 1024:.2f} MB")
        except subprocess.CalledProcessError as e:
            print(f"Failed to create image: {e}")
            sys.exit(1)
        except FileNotFoundError:
            print("Error: grub-mkrescue not found. Install grub2.")
            sys.exit(1)
            
    def build(self):
        """Build complete image"""
        print("Building Hexphyr OS image...")
        self.prepare_directories()
        self.copy_files()
        self.create_image()
        print("Done!")

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Build Hexphyr OS image")
    parser.add_argument("--output", "-o", default="hexphyr.iso",
                       help="Output ISO filename")
    
    args = parser.parse_args()
    
    builder = ImageBuilder(args.output)
    builder.build()