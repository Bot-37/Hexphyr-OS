#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use bootabi::{
    BootInfo, FramebufferInfo, MemoryMapInfo, BOOT_FLAG_ACPI_RSDP,
    BOOT_FLAG_FRAMEBUFFER, BOOT_FLAG_INITRAMFS, BOOT_FLAG_MEMORY_MAP,
    BOOT_FLAG_UEFI, BOOTINFO_REVISION, PIXEL_FORMAT_BGR, PIXEL_FORMAT_BITMASK,
    PIXEL_FORMAT_RGB, UefiMemoryDescriptor,
};
use core::fmt::Write;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::{copy_nonoverlapping, read_unaligned, write_bytes};
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelBitmask, PixelFormat};
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType};
use uefi::table::cfg;

const KERNEL_PATH: &uefi::CStr16 = cstr16!(r"\EFI\HEXPHYR\KERNEL.ELF");
const INITRAMFS_PATH: &uefi::CStr16 = cstr16!(r"\EFI\HEXPHYR\INITRAMFS.BIN");
const VERSION_PATH: &uefi::CStr16 = cstr16!(r"\EFI\HEXPHYR\VERSION.TXT");

const ELF_MAGIC: &[u8; 4] = b"\x7fELF";
const ELF_CLASS_64: u8 = 2;
const ELF_DATA_LSB: u8 = 1;
const ELF_TYPE_EXEC: u16 = 2;
const ELF_MACHINE_X86_64: u16 = 62;
const PT_LOAD: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const PAGE_SIZE: u64 = 4096;

type KernelEntry = unsafe extern "sysv64" fn(*const BootInfo) -> !;

#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64Header {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64ProgramHeader {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64SectionHeader {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64Symbol {
    st_name: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
    st_value: u64,
    st_size: u64,
}

struct KernelImage {
    entry: KernelEntry,
}

static mut BOOT_INFO: BootInfo = BootInfo::empty();

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    unsafe {
        uefi::allocator::init(&mut system_table);
    }
    let _ = system_table.stdout().reset(false);

    match boot(image_handle, system_table) {
        Ok(()) => Status::SUCCESS,
        Err(status) => status,
    }
}

fn boot(image_handle: Handle, mut system_table: SystemTable<Boot>) -> core::result::Result<(), Status> {
    write_line(&mut system_table, "Hexphyr UEFI bootloader");

    write_line(&mut system_table, "Reading kernel image");
    let kernel_bytes = read_file(system_table.boot_services(), image_handle, KERNEL_PATH)?;
    write_line(&mut system_table, "Reading initramfs");
    let initramfs_bytes =
        read_optional_file(system_table.boot_services(), image_handle, INITRAMFS_PATH)?;
    let _ = read_optional_file(system_table.boot_services(), image_handle, VERSION_PATH)?;

    write_line(&mut system_table, "Loading kernel ELF");
    let kernel_image = load_kernel_image(system_table.boot_services(), &kernel_bytes)?;
    write_line(&mut system_table, "Locating GOP framebuffer");
    let framebuffer = locate_framebuffer(system_table.boot_services())?;
    let rsdp_addr = find_rsdp(&system_table);

    let initramfs_addr = initramfs_bytes
        .as_ref()
        .map(|bytes| bytes.as_ptr() as u64)
        .unwrap_or(0);
    let initramfs_size = initramfs_bytes
        .as_ref()
        .map(|bytes| bytes.len() as u64)
        .unwrap_or(0);

    write_line(&mut system_table, "Exiting boot services");
    uefi::allocator::exit_boot_services();
    let (_runtime_table, memory_map) = system_table.exit_boot_services(MemoryType::LOADER_DATA);

    let memory_map_info = build_memory_map_info(&memory_map);
    let mut flags = BOOT_FLAG_UEFI | BOOT_FLAG_FRAMEBUFFER | BOOT_FLAG_MEMORY_MAP;
    if initramfs_addr != 0 && initramfs_size != 0 {
        flags |= BOOT_FLAG_INITRAMFS;
    }
    if rsdp_addr != 0 {
        flags |= BOOT_FLAG_ACPI_RSDP;
    }

    unsafe {
        BOOT_INFO = BootInfo {
            revision: BOOTINFO_REVISION,
            size: size_of::<BootInfo>() as u32,
            flags,
            framebuffer,
            memory_map: memory_map_info,
            initramfs_addr,
            initramfs_size,
            rsdp_addr,
            reserved: 0,
        };

        (kernel_image.entry)(&raw const BOOT_INFO);
    }
}

fn write_line(system_table: &mut SystemTable<Boot>, message: &str) {
    let _ = writeln!(system_table.stdout(), "{message}");
}

fn read_file(
    boot_services: &uefi::table::boot::BootServices,
    image_handle: Handle,
    path: &uefi::CStr16,
) -> core::result::Result<Vec<u8>, Status> {
    let mut fs = boot_services
        .get_image_file_system(image_handle)
        .map_err(|err| err.status())?;
    let mut root = fs.open_volume().map_err(|err| err.status())?;
    let handle = root
        .open(path, FileMode::Read, FileAttribute::empty())
        .map_err(|err| err.status())?;
    let mut file = match handle.into_type().map_err(|err| err.status())? {
        FileType::Regular(file) => file,
        _ => return Err(Status::UNSUPPORTED),
    };

    let info = file
        .get_boxed_info::<FileInfo>()
        .map_err(|err| err.status())?;
    let file_size = info.file_size() as usize;
    let mut bytes = vec![0u8; file_size];
    let read_len = file.read(&mut bytes).map_err(|err| err.status())?;
    bytes.truncate(read_len);

    Ok(bytes)
}

fn read_optional_file(
    boot_services: &uefi::table::boot::BootServices,
    image_handle: Handle,
    path: &uefi::CStr16,
) -> core::result::Result<Option<Vec<u8>>, Status> {
    match read_file(boot_services, image_handle, path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(Status::NOT_FOUND) => Ok(None),
        Err(status) => Err(status),
    }
}

fn load_kernel_image(
    boot_services: &uefi::table::boot::BootServices,
    kernel_bytes: &[u8],
) -> core::result::Result<KernelImage, Status> {
    let header = validate_elf_header(kernel_bytes)?;
    let (load_start, load_end) = load_range(kernel_bytes, &header)?;

    let page_count = ((load_end - load_start) / PAGE_SIZE) as usize;
    boot_services
        .allocate_pages(
            AllocateType::Address(load_start),
            MemoryType::LOADER_DATA,
            page_count,
        )
        .map_err(|err| err.status())?;

    unsafe {
        write_bytes(load_start as *mut u8, 0, (load_end - load_start) as usize);
    }

    for index in 0..usize::from(header.e_phnum) {
        let ph = read_program_header(kernel_bytes, &header, index)?;
        if ph.p_type != PT_LOAD {
            continue;
        }

        if ph.p_filesz > ph.p_memsz {
            return Err(Status::LOAD_ERROR);
        }

        let file_start = ph.p_offset as usize;
        let file_end = file_start
            .checked_add(ph.p_filesz as usize)
            .ok_or(Status::LOAD_ERROR)?;
        if file_end > kernel_bytes.len() {
            return Err(Status::LOAD_ERROR);
        }

        let target_addr = segment_addr(&ph) as *mut u8;
        unsafe {
            copy_nonoverlapping(
                kernel_bytes.as_ptr().add(file_start),
                target_addr,
                ph.p_filesz as usize,
            );
        }
    }

    let entry_addr = resolve_symbol(kernel_bytes, &header, b"_uefi_start")
        .unwrap_or(header.e_entry);
    let entry = unsafe { core::mem::transmute::<usize, KernelEntry>(entry_addr as usize) };

    Ok(KernelImage { entry })
}

fn locate_framebuffer(
    boot_services: &uefi::table::boot::BootServices,
) -> core::result::Result<FramebufferInfo, Status> {
    let handle = boot_services
        .get_handle_for_protocol::<GraphicsOutput>()
        .map_err(|err| err.status())?;
    let mut gop = boot_services
        .open_protocol_exclusive::<GraphicsOutput>(handle)
        .map_err(|err| err.status())?;

    let mode_info = gop.current_mode_info();
    let (width, height) = mode_info.resolution();
    let mut framebuffer = gop.frame_buffer();

    let (pixel_format, red_pos, red_size, green_pos, green_size, blue_pos, blue_size) =
        match mode_info.pixel_format() {
            PixelFormat::Rgb => (PIXEL_FORMAT_RGB, 0, 8, 8, 8, 16, 8),
            PixelFormat::Bgr => (PIXEL_FORMAT_BGR, 16, 8, 8, 8, 0, 8),
            PixelFormat::Bitmask => bitmask_layout(
                mode_info.pixel_bitmask().ok_or(Status::UNSUPPORTED)?,
            ),
            PixelFormat::BltOnly => return Err(Status::UNSUPPORTED),
        };

    Ok(FramebufferInfo {
        address: framebuffer.as_mut_ptr() as u64,
        size: framebuffer.size() as u64,
        width: width as u32,
        height: height as u32,
        pitch: (mode_info.stride() * 4) as u32,
        bpp: 32,
        pixel_format,
        red_field_position: red_pos,
        red_mask_size: red_size,
        green_field_position: green_pos,
        green_mask_size: green_size,
        blue_field_position: blue_pos,
        blue_mask_size: blue_size,
        reserved: 0,
    })
}

fn bitmask_layout(mask: PixelBitmask) -> (u8, u8, u8, u8, u8, u8, u8) {
    let (red_pos, red_size) = mask_parts(mask.red);
    let (green_pos, green_size) = mask_parts(mask.green);
    let (blue_pos, blue_size) = mask_parts(mask.blue);

    (
        PIXEL_FORMAT_BITMASK,
        red_pos,
        red_size,
        green_pos,
        green_size,
        blue_pos,
        blue_size,
    )
}

fn mask_parts(mask: u32) -> (u8, u8) {
    if mask == 0 {
        return (0, 0);
    }

    (mask.trailing_zeros() as u8, mask.count_ones() as u8)
}

fn find_rsdp(system_table: &SystemTable<Boot>) -> u64 {
    system_table
        .config_table()
        .iter()
        .find(|entry| entry.guid == cfg::ACPI2_GUID)
        .or_else(|| {
            system_table
                .config_table()
                .iter()
                .find(|entry| entry.guid == cfg::ACPI_GUID)
        })
        .map(|entry| entry.address as u64)
        .unwrap_or(0)
}

fn build_memory_map_info(memory_map: &uefi::table::boot::MemoryMap<'_>) -> MemoryMapInfo {
    let entry_count = memory_map.entries().len() as u32;
    let address = memory_map
        .get(0)
        .map(|entry| entry as *const MemoryDescriptor as u64)
        .unwrap_or(0);

    MemoryMapInfo {
        address,
        entry_count,
        entry_size: size_of::<UefiMemoryDescriptor>() as u32,
        descriptor_version: MemoryDescriptor::VERSION,
        reserved: 0,
    }
}

fn validate_elf_header(bytes: &[u8]) -> core::result::Result<Elf64Header, Status> {
    let header = read_struct::<Elf64Header>(bytes, 0).ok_or(Status::LOAD_ERROR)?;

    if &header.e_ident[..4] != ELF_MAGIC
        || header.e_ident[4] != ELF_CLASS_64
        || header.e_ident[5] != ELF_DATA_LSB
        || header.e_type != ELF_TYPE_EXEC
        || header.e_machine != ELF_MACHINE_X86_64
        || header.e_phentsize as usize != size_of::<Elf64ProgramHeader>()
        || header.e_shentsize as usize != size_of::<Elf64SectionHeader>()
    {
        return Err(Status::LOAD_ERROR);
    }

    Ok(header)
}

fn load_range(bytes: &[u8], header: &Elf64Header) -> core::result::Result<(u64, u64), Status> {
    let mut min_addr = u64::MAX;
    let mut max_addr = 0u64;

    for index in 0..usize::from(header.e_phnum) {
        let ph = read_program_header(bytes, header, index)?;
        if ph.p_type != PT_LOAD {
            continue;
        }

        let seg_addr = segment_addr(&ph);
        min_addr = min_addr.min(align_down(seg_addr, PAGE_SIZE));
        max_addr = max_addr.max(align_up(seg_addr + ph.p_memsz, PAGE_SIZE));
    }

    if min_addr == u64::MAX || max_addr <= min_addr {
        return Err(Status::LOAD_ERROR);
    }

    Ok((min_addr, max_addr))
}

fn resolve_symbol(
    bytes: &[u8],
    header: &Elf64Header,
    symbol_name: &[u8],
) -> Option<u64> {
    for section_index in 0..usize::from(header.e_shnum) {
        let section = read_section_header(bytes, header, section_index)?;
        if section.sh_type != SHT_SYMTAB || section.sh_entsize == 0 {
            continue;
        }

        let strtab = read_section_header(bytes, header, section.sh_link as usize)?;
        let symbol_count = (section.sh_size / section.sh_entsize) as usize;

        for symbol_index in 0..symbol_count {
            let offset = section.sh_offset as usize
                + symbol_index * section.sh_entsize as usize;
            let symbol = read_struct::<Elf64Symbol>(bytes, offset)?;
            let name = read_c_string(bytes, strtab.sh_offset as usize + symbol.st_name as usize)?;
            if name == symbol_name {
                return Some(symbol.st_value);
            }
        }
    }

    None
}

fn read_program_header(
    bytes: &[u8],
    header: &Elf64Header,
    index: usize,
) -> core::result::Result<Elf64ProgramHeader, Status> {
    let offset = header.e_phoff as usize
        + index
            .checked_mul(size_of::<Elf64ProgramHeader>())
            .ok_or(Status::LOAD_ERROR)?;
    read_struct::<Elf64ProgramHeader>(bytes, offset).ok_or(Status::LOAD_ERROR)
}

fn read_section_header(
    bytes: &[u8],
    header: &Elf64Header,
    index: usize,
) -> Option<Elf64SectionHeader> {
    let offset = header.e_shoff as usize
        + index.checked_mul(size_of::<Elf64SectionHeader>())?;
    read_struct::<Elf64SectionHeader>(bytes, offset)
}

fn read_c_string(bytes: &[u8], offset: usize) -> Option<&[u8]> {
    let tail = bytes.get(offset..)?;
    let nul = tail.iter().position(|byte| *byte == 0)?;
    Some(&tail[..nul])
}

fn read_struct<T: Copy>(bytes: &[u8], offset: usize) -> Option<T> {
    let end = offset.checked_add(size_of::<T>())?;
    let src = bytes.get(offset..end)?;

    Some(unsafe { read_unaligned(src.as_ptr().cast::<T>()) })
}

fn segment_addr(program_header: &Elf64ProgramHeader) -> u64 {
    if program_header.p_paddr != 0 {
        program_header.p_paddr
    } else {
        program_header.p_vaddr
    }
}

fn align_down(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}

fn align_up(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        // SAFETY: `hlt` is valid in firmware context and keeps the CPU quiescent.
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) }
    }
}
