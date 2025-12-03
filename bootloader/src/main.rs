#![no_std]
#![no_main]
#![feature(asm_const)]

use core::panic::PanicInfo;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileMode, FileType};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::CStr16;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn efi_main(image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&system_table).expect("Failed to initialize UEFI services");
    
    log::info!("Hexphyr Bootloader v0.1");
    
    // Get the boot volume
    let boot_services = system_table.boot_services();
    let sfs = boot_services
        .get_image_file_system(image_handle)
        .expect("Failed to get file system");
    let mut volume = sfs.open_volume().expect("Failed to open volume");
    
    // Look for kernel
    const KERNEL_NAME: &CStr16 = cstr16!("kernel.elf");
    match load_kernel(&mut volume, boot_services, KERNEL_NAME) {
        Ok(entry_point) => {
            log::info!("Kernel loaded at {:p}, jumping...", entry_point);
            
            // Exit boot services before jumping to kernel
            let (_st, mmap) = system_table
                .exit_boot_services()
                .expect("Failed to exit boot services");
            
            // Jump to kernel
            unsafe {
                core::arch::asm!(
                    "mov rdi, {0:x}",
                    "call {1:x}",
                    in(reg) &mmap as *const _,
                    in(reg) entry_point,
                    options(noreturn)
                );
            }
        }
        Err(e) => {
            log::error!("Failed to load kernel: {:?}", e);
            Status::LOAD_ERROR
        }
    }
}

fn load_kernel(
    volume: &mut File,
    boot_services: &BootServices,
    filename: &CStr16,
) -> Result<*const u8, uefi::Error> {
    // Open kernel file
    let mut file = volume
        .open(filename, FileMode::Read, FileAttribute::empty())?
        .into_type()?;
    
    let FileType::Regular(mut file) = file else {
        return Err(uefi::Error::from(Status::UNSUPPORTED));
    };
    
    // Get file size
    let file_info = file.get_boxed_info::<uefi::proto::media::file::FileInfo>()?;
    let file_size = file_info.file_size() as usize;
    
    // Allocate memory for kernel (1MB for now)
    const KERNEL_BASE: u64 = 0x100000;
    let pages = (file_size + 0xFFF) / 0x1000; // Round up to pages
    boot_services.allocate_pages(
        AllocateType::Address(KERNEL_BASE),
        MemoryType::LOADER_DATA,
        pages,
    )?;
    
    // Read kernel into memory
    let kernel_slice = unsafe {
        core::slice::from_raw_parts_mut(KERNEL_BASE as *mut u8, file_size)
    };
    file.read(kernel_slice)?;
    
    // TODO: Parse ELF header and get actual entry point
    // For now, assume flat binary
    Ok(KERNEL_BASE as *const u8)
}