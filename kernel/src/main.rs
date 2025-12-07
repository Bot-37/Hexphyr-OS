// src/main.rs
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::panic::PanicInfo;
use log::{info, error};

mod serial;
mod memory;
mod interrupts;
mod arch;
mod multiboot2_header;

use x86_64::{VirtAddr, PhysAddr};

use arch::x86_64::paging;

/// Updated BootInfo with kernel start/end physical addresses
#[repr(C)]
pub struct BootInfo {
    pub memory_map: *const u8,
    pub map_size: usize,
    pub descriptor_size: usize,
    pub descriptor_version: u32,
    pub kernel_start_phys: u64,
    pub kernel_end_phys: u64,
}

/// Kernel entry point
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    // init serial early so logs work
    serial::init();
    // register the logger
    log::set_logger(&serial::LOGGER).ok();
    log::set_max_level(log::LevelFilter::Info);

    info!("Hexphyr Kernel booting");
    info!("BootInfo at {:p}", boot_info);
    info!("Kernel phys: {:#x} - {:#x}", boot_info.kernel_start_phys, boot_info.kernel_end_phys);

    // Initialize memory: heap + frame allocator
    let (frame_alloc, heap_start, heap_size) = memory::init(boot_info);
    info!("Heap at {:#x}, size {} KiB", heap_start, heap_size / 1024);
    info!("Frame allocator prepared (static)");

    // Build a FrameAllocatorRef wrapper to pass to paging functions
    let mut fa_ref = memory::FrameAllocatorRef::new(frame_alloc);

    // Choose higher-half physical memory offset (recommended)
    let physical_memory_offset = VirtAddr::new(0xffff_8000_0000_0000u64);

    // Initialize OffsetPageTable for the current active page tables
    let mut mapper = unsafe { paging::init(physical_memory_offset) };
    info!("OffsetPageTable created using physical_memory_offset = {:#x}", physical_memory_offset.as_u64());

    // Map the kernel image into the higher half virtual addresses
    let kernel_phys_start = PhysAddr::new(boot_info.kernel_start_phys);
    let kernel_phys_end = PhysAddr::new(boot_info.kernel_end_phys);
    paging::map_phys_to_virt_range(&mut mapper, &mut fa_ref, kernel_phys_start, kernel_phys_end, physical_memory_offset, paging::kernel_flags())
    .expect("Failed to map kernel into higher-half");

    info!("Kernel mapped into higher-half");

    // Map the kernel heap region (phys->virt at offset)
    let heap_phys_start = PhysAddr::new(heap_start as u64);
    let heap_phys_end = PhysAddr::new((heap_start + heap_size) as u64);
    paging::map_phys_to_virt_range(&mut mapper, &mut fa_ref, heap_phys_start, heap_phys_end, physical_memory_offset, paging::kernel_flags())
    .expect("Failed to map heap into higher-half");

    info!("Heap mapped into higher-half");

    // Initialize interrupts + IDT and enable interrupts
    interrupts::init();
    interrupts::enable();
    info!("Interrupts initialized and enabled");

    // Test allocation (Box/Vec) now that heap is mapped & inited
    let boxed = alloc::boxed::Box::new(123u64);
    info!("Box test succeeded, val = {}", *boxed);

    // Enter the main loop
    main_loop();
}

fn main_loop() -> ! {
    let mut counter: u64 = 0;
    loop {
        info!("Kernel alive tick {}", counter);
        counter = counter.wrapping_add(1);
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("KERNEL PANIC: {}", info);
    interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn oom(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
