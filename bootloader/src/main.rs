// Hexphyr OS — UEFI Bootloader
//
// This crate is compiled as a UEFI application (x86_64-unknown-uefi target).
// It is loaded by UEFI firmware, initialises UEFI services, and is responsible
// for loading the Hexphyr kernel ELF from the EFI System Partition and
// transferring control to it.
//
// Current status: skeleton.  The kernel is currently loaded via the GRUB
// Multiboot2 path (grub.cfg → kernel.elf) rather than through this bootloader.
// This crate is the foundation for a native UEFI boot path.

#![no_std]
#![no_main]

use uefi::prelude::*;

/// UEFI entry point.
///
/// `uefi-services` provides the global allocator, structured logger, and
/// panic handler — do NOT define a custom `#[panic_handler]` in this crate.
#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialise UEFI services");

    log::info!(
        "Hexphyr UEFI bootloader v{}",
        env!("CARGO_PKG_VERSION")
    );

    // -----------------------------------------------------------------------
    // TODO: Locate the EFI System Partition via the Loaded Image Protocol.
    // TODO: Open the Simple File System Protocol on the boot volume.
    // TODO: Read and validate \EFI\Hexphyr\kernel.elf (ELF64 header check).
    // TODO: Allocate EfiLoaderData pages for each PT_LOAD segment.
    // TODO: Copy segments and zero BSS ranges.
    // TODO: Retrieve the UEFI memory map via GetMemoryMap.
    // TODO: Call ExitBootServices (invalidates system_table boot services).
    // TODO: Jump to the kernel entry point, passing the memory map pointer.
    // -----------------------------------------------------------------------

    log::error!("Kernel handoff not yet implemented — halting");

    loop {
        // Busy-wait HLT keeps the CPU in a low-power state.
        // SAFETY: `hlt` is a privileged instruction that is safe to execute
        // in kernel/firmware context and has no side-effects visible to Rust.
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) }
    }
}

