#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;
extern crate lazy_static;

use core::panic::PanicInfo;

mod interrupts;
mod memory;
mod serial;
mod arch {
    pub mod x86_64 {
        pub mod paging;
    }
}

// FIX: rename the import to avoid duplicate module name
use x86_64::instructions::interrupts as cpu_interrupts;

#[no_mangle]
pub extern "C" fn _start(_boot_info_ptr: *const u8) -> ! {
    serial::init_serial();
    interrupts::init_idt();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
