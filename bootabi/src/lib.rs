#![no_std]

pub const BOOTINFO_REVISION: u32 = 1;

pub const BOOT_FLAG_UEFI: u64 = 1 << 0;
pub const BOOT_FLAG_MULTIBOOT: u64 = 1 << 1;
pub const BOOT_FLAG_FRAMEBUFFER: u64 = 1 << 2;
pub const BOOT_FLAG_MEMORY_MAP: u64 = 1 << 3;
pub const BOOT_FLAG_INITRAMFS: u64 = 1 << 4;
pub const BOOT_FLAG_ACPI_RSDP: u64 = 1 << 5;

pub const PIXEL_FORMAT_RGB: u8 = 1;
pub const PIXEL_FORMAT_BGR: u8 = 2;
pub const PIXEL_FORMAT_BITMASK: u8 = 3;
pub const PIXEL_FORMAT_UNKNOWN: u8 = 255;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FramebufferInfo {
    pub address: u64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u8,
    pub pixel_format: u8,
    pub red_field_position: u8,
    pub red_mask_size: u8,
    pub green_field_position: u8,
    pub green_mask_size: u8,
    pub blue_field_position: u8,
    pub blue_mask_size: u8,
    pub reserved: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryMapInfo {
    pub address: u64,
    pub entry_count: u32,
    pub entry_size: u32,
    pub descriptor_version: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct BootInfo {
    pub revision: u32,
    pub size: u32,
    pub flags: u64,
    pub framebuffer: FramebufferInfo,
    pub memory_map: MemoryMapInfo,
    pub initramfs_addr: u64,
    pub initramfs_size: u64,
    pub rsdp_addr: u64,
    pub reserved: u64,
}

impl BootInfo {
    pub const fn empty() -> Self {
        Self {
            revision: BOOTINFO_REVISION,
            size: core::mem::size_of::<Self>() as u32,
            flags: 0,
            framebuffer: FramebufferInfo {
                address: 0,
                size: 0,
                width: 0,
                height: 0,
                pitch: 0,
                bpp: 0,
                pixel_format: PIXEL_FORMAT_UNKNOWN,
                red_field_position: 0,
                red_mask_size: 0,
                green_field_position: 0,
                green_mask_size: 0,
                blue_field_position: 0,
                blue_mask_size: 0,
                reserved: 0,
            },
            memory_map: MemoryMapInfo {
                address: 0,
                entry_count: 0,
                entry_size: 0,
                descriptor_version: 0,
                reserved: 0,
            },
            initramfs_addr: 0,
            initramfs_size: 0,
            rsdp_addr: 0,
            reserved: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct UefiMemoryDescriptor {
    pub ty: u32,
    pub padding: u32,
    pub phys_start: u64,
    pub virt_start: u64,
    pub page_count: u64,
    pub att: u64,
}
