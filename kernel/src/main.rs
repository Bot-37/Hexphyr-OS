#![no_std]
#![no_main]

use core::{hint::spin_loop, panic::PanicInfo};
use log::{error, info};

mod gui;
mod multiboot;
mod multiboot2_header;
mod serial;

core::arch::global_asm!(include_str!("boot.s"), options(att_syntax));

use gui::Framebuffer;
use multiboot::MultibootInfo;

#[no_mangle]
pub extern "C" fn _start(multiboot_info_addr: u64) -> ! {
    serial::init();
    log::set_logger(&serial::LOGGER).ok();
    log::set_max_level(log::LevelFilter::Info);

    info!("Hexphyr kernel booting");
    info!("Multiboot info pointer: {:#x}", multiboot_info_addr);

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

    info!(
        "Framebuffer: {}x{} {}bpp pitch={} addr={:#x}",
        framebuffer_info.width,
        framebuffer_info.height,
        framebuffer_info.bpp,
        framebuffer_info.pitch,
        framebuffer_info.address
    );

    let Some(mut framebuffer) = Framebuffer::new(framebuffer_info) else {
        error!("Framebuffer parameters are not supported by this renderer");
        halt_forever()
    };

    main_loop(&mut framebuffer);
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
