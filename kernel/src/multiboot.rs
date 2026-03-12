use core::mem;

const TAG_TYPE_END: u32 = 0;
const TAG_TYPE_FRAMEBUFFER: u32 = 8;

#[derive(Clone, Copy, Debug)]
pub struct FramebufferInfo {
    pub address: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub buffer_type: u8,
    pub red_field_position: u8,
    pub red_mask_size: u8,
    pub green_field_position: u8,
    pub green_mask_size: u8,
    pub blue_field_position: u8,
    pub blue_mask_size: u8,
}

#[derive(Clone, Copy)]
pub struct MultibootInfo {
    base: usize,
    total_size: usize,
}

#[repr(C)]
struct TagHeader {
    tag_type: u32,
    size: u32,
}

impl MultibootInfo {
    pub unsafe fn new(base: usize) -> Option<Self> {
        if base == 0 {
            return None;
        }

        let total_size = unsafe { *(base as *const u32) as usize };
        if total_size < 8 {
            return None;
        }

        Some(Self { base, total_size })
    }

    pub fn total_size(&self) -> usize {
        self.total_size
    }

    pub fn framebuffer(&self) -> Option<FramebufferInfo> {
        let mut offset = 8usize;

        while offset + mem::size_of::<TagHeader>() <= self.total_size {
            let tag_ptr = (self.base + offset) as *const u8;
            let header = unsafe { &*(tag_ptr as *const TagHeader) };
            let tag_size = header.size as usize;

            if header.tag_type == TAG_TYPE_END {
                break;
            }

            if tag_size < mem::size_of::<TagHeader>() || offset + tag_size > self.total_size {
                break;
            }

            if header.tag_type == TAG_TYPE_FRAMEBUFFER {
                return parse_framebuffer_tag(tag_ptr, tag_size);
            }

            offset += align_up_8(tag_size);
        }

        None
    }
}

fn parse_framebuffer_tag(tag_ptr: *const u8, tag_size: usize) -> Option<FramebufferInfo> {
    // Framebuffer tag common fields occupy 32 bytes.
    if tag_size < 32 {
        return None;
    }

    let address = unsafe { *(tag_ptr.add(8) as *const u64) };
    let pitch = unsafe { *(tag_ptr.add(16) as *const u32) };
    let width = unsafe { *(tag_ptr.add(20) as *const u32) };
    let height = unsafe { *(tag_ptr.add(24) as *const u32) };
    let bpp = unsafe { *tag_ptr.add(28) };
    let buffer_type = unsafe { *tag_ptr.add(29) };

    // Default bit positions for common 32bpp XRGB mode in QEMU.
    let mut red_field_position = 16;
    let mut red_mask_size = 8;
    let mut green_field_position = 8;
    let mut green_mask_size = 8;
    let mut blue_field_position = 0;
    let mut blue_mask_size = 8;

    // For direct RGB framebuffers, multiboot includes channel layout bytes.
    if buffer_type == 1 && tag_size >= 38 {
        red_field_position = unsafe { *tag_ptr.add(32) };
        red_mask_size = unsafe { *tag_ptr.add(33) };
        green_field_position = unsafe { *tag_ptr.add(34) };
        green_mask_size = unsafe { *tag_ptr.add(35) };
        blue_field_position = unsafe { *tag_ptr.add(36) };
        blue_mask_size = unsafe { *tag_ptr.add(37) };
    }

    Some(FramebufferInfo {
        address,
        pitch,
        width,
        height,
        bpp,
        buffer_type,
        red_field_position,
        red_mask_size,
        green_field_position,
        green_mask_size,
        blue_field_position,
        blue_mask_size,
    })
}

fn align_up_8(value: usize) -> usize {
    (value + 7) & !7
}
