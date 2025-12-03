#![no_std]
#![no_main]
#![feature(asm_const, naked_functions, abi_x86_interrupt)]

mod serial;
mod memory;
mod interrupts;

use core::arch::global_asm;
use core::panic::PanicInfo;
use log::{info, error};
use x86_64::instructions::interrupts;

/// UEFI memory map structure passed by bootloader
#[repr(C)]
pub struct BootInfo {
    pub memory_map: *const u8,
    pub map_size: usize,
    pub descriptor_size: usize,
    pub descriptor_version: u32,
}

/// Kernel entry point (called by bootloader with RDI = BootInfo*)
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    // Initialize serial output first
    serial::init();
    
    // Set up logging
    log::set_logger(&serial::LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    
    info!("Hexphyr Kernel v0.1");
    info!("Boot info at: {:p}", boot_info);
    
    // Initialize memory manager
    memory::init(boot_info);
    
    // Set up interrupt descriptor table
    interrupts::init();
    
    // Enable interrupts
    interrupts::enable();
    
    info!("Kernel initialized. Entering main loop.");
    
    // Main kernel loop
    main_loop();
}

fn main_loop() -> ! {
    let mut counter = 0;
    loop {
        interrupts::disable();
        info!("Kernel alive: tick {}", counter);
        counter += 1;
        interrupts::enable();
        
        // Halt until next interrupt
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("KERNEL PANIC: {}", info);
    
    // Disable interrupts and halt
    interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

global_asm!(include_str!("boot.s"));