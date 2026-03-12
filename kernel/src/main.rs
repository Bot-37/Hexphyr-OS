#![no_std]
#![no_main]
// Required for `extern "x86-interrupt" fn` in interrupts.rs / gdt.rs.
#![feature(abi_x86_interrupt)]

use bootabi::{
    BootInfo, FramebufferInfo, BOOT_FLAG_ACPI_RSDP, BOOT_FLAG_FRAMEBUFFER,
    BOOT_FLAG_INITRAMFS, BOOT_FLAG_MEMORY_MAP, BOOT_FLAG_MULTIBOOT,
    BOOT_FLAG_UEFI,
};
use core::{hint::spin_loop, panic::PanicInfo};
use log::{error, info};

mod gdt;
mod gui;
mod initramfs;
mod interrupts;
mod memory;
mod multiboot;
mod multiboot2_header;
mod serial;

core::arch::global_asm!(include_str!("boot.s"), options(att_syntax));

use gui::Framebuffer;
use multiboot::MultibootInfo;

#[used]
static KEEP_UEFI_ENTRY: unsafe extern "C" fn(*const BootInfo) -> ! = _uefi_start;

#[no_mangle]
pub extern "C" fn _start(multiboot_info_addr: u64) -> ! {
    init_runtime();

    info!("Hexphyr OS — Multiboot kernel entry");
    info!("Multiboot2 info pointer: {:#x}", multiboot_info_addr);

    let multiboot_info = unsafe { MultibootInfo::new(multiboot_info_addr as usize) };
    let Some(multiboot_info) = multiboot_info else {
        error!("Invalid Multiboot2 information pointer");
        halt_forever()
    };

    info!(
        "Multiboot2 total size: {} bytes",
        multiboot_info.total_size()
    );

    let Some(framebuffer_info) = multiboot_info.framebuffer() else {
        error!("No framebuffer tag found. GUI cannot be initialized.");
        halt_forever()
    };

    let mut boot_info = BootInfo::empty();
    boot_info.flags = BOOT_FLAG_MULTIBOOT | BOOT_FLAG_FRAMEBUFFER;
    boot_info.framebuffer = framebuffer_info;

    kernel_entry(&boot_info)
}

#[no_mangle]
pub extern "C" fn _uefi_start(boot_info: *const BootInfo) -> ! {
    init_runtime();

    if boot_info.is_null() {
        error!("UEFI bootloader passed a null BootInfo pointer");
        halt_forever()
    }

    let boot_info = unsafe { &*boot_info };
    info!("Hexphyr OS — UEFI kernel entry");
    kernel_entry(boot_info)
}

fn init_runtime() {
    // Serial must be first so every subsequent log call has an output channel.
    serial::init();
    log::set_logger(&serial::LOGGER).ok();
    log::set_max_level(log::LevelFilter::Info);

    // Initialize the production GDT (with TSS/IST) BEFORE loading the IDT so
    // that the double-fault IST stack index references a valid TSS.
    gdt::init();
    info!("GDT loaded");

    // Load the IDT.  From this point onward CPU exceptions are handled.
    interrupts::init();
    info!("IDT loaded");
}

fn kernel_entry(boot_info: &BootInfo) -> ! {
    log_boot_info(boot_info);
    log_initramfs(boot_info);

    let Some(mut framebuffer) = Framebuffer::new(boot_info.framebuffer) else {
        error!("Framebuffer parameters are not supported by this renderer");
        halt_forever()
    };

    main_loop(&mut framebuffer);
}

fn log_boot_info(boot_info: &BootInfo) {
    info!(
        "BootInfo rev={} size={} flags={:#x}",
        boot_info.revision,
        boot_info.size,
        boot_info.flags
    );

    if has_flag(boot_info.flags, BOOT_FLAG_UEFI) {
        info!("Boot source: UEFI");
    }
    if has_flag(boot_info.flags, BOOT_FLAG_MULTIBOOT) {
        info!("Boot source: Multiboot2");
    }
    if has_flag(boot_info.flags, BOOT_FLAG_FRAMEBUFFER) {
        log_framebuffer(boot_info.framebuffer);
    }
    if has_flag(boot_info.flags, BOOT_FLAG_MEMORY_MAP) {
        info!(
            "Memory map: ptr={:#x} entries={} entry_size={} version={}",
            boot_info.memory_map.address,
            boot_info.memory_map.entry_count,
            boot_info.memory_map.entry_size,
            boot_info.memory_map.descriptor_version
        );
    }
    if has_flag(boot_info.flags, BOOT_FLAG_INITRAMFS) {
        info!(
            "Initramfs: addr={:#x} size={} bytes",
            boot_info.initramfs_addr,
            boot_info.initramfs_size
        );
    }
    if has_flag(boot_info.flags, BOOT_FLAG_ACPI_RSDP) {
        info!("ACPI RSDP: {:#x}", boot_info.rsdp_addr);
    }
}

fn log_framebuffer(framebuffer: FramebufferInfo) {
    info!(
        "Framebuffer: {}x{} {}bpp pitch={} size={} addr={:#x}",
        framebuffer.width,
        framebuffer.height,
        framebuffer.bpp,
        framebuffer.pitch,
        framebuffer.size,
        framebuffer.address
    );
}

fn has_flag(flags: u64, flag: u64) -> bool {
    flags & flag != 0
}

fn log_initramfs(boot_info: &BootInfo) {
    if !has_flag(boot_info.flags, BOOT_FLAG_INITRAMFS) {
        return;
    }

    let archive = unsafe {
        initramfs::Archive::from_raw(
            boot_info.initramfs_addr as *const u8,
            boot_info.initramfs_size as usize,
        )
    };
    let Some(archive) = archive else {
        error!("Initramfs pointer or size is invalid");
        return;
    };

    let summary = archive.summary();
    info!(
        "Initramfs entries={} bytes={} has_init={} has_shell={} has_issue={}",
        summary.entry_count,
        summary.payload_bytes,
        summary.has_init,
        summary.has_shell,
        summary.has_issue
    );

    if !summary.has_init {
        error!("Initramfs is missing /sbin/init");
    }
    if !summary.has_shell {
        error!("Initramfs is missing /bin/sh");
    }
    if !summary.has_issue {
        error!("Initramfs is missing /etc/issue");
    }
}

fn main_loop(framebuffer: &mut Framebuffer) -> ! {
    let mut counter: u64 = 0;
    loop {
        gui::draw_desktop(framebuffer, counter);
        if counter % 120 == 0 {
            info!("GUI tick {}", counter);
        }
        counter = counter.wrapping_add(1);
        delay_cycles(2_500_000);
    }
}

fn delay_cycles(cycles: usize) {
    for _ in 0..cycles {
        spin_loop();
    }
}

fn halt_forever() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("KERNEL PANIC: {}", info);
    halt_forever()
}
