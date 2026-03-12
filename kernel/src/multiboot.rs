// kernel/src/multiboot.rs
//
// Minimal Multiboot2 information-structure parser.
//
// Security / correctness notes:
//   * ALL multi-byte reads from raw pointer offsets use `ptr::read_unaligned`
//     to eliminate undefined behaviour.  The Multiboot2 spec guarantees 8-byte
//     alignment for the info structure, but individual tag fields are NOT
//     guaranteed to be aligned after arbitrary tag sizes, so read_unaligned is
//     always correct here.
//   * The info pointer itself is validated for 8-byte alignment and a non-zero
//     value before any dereference.
//   * The reserved word (bytes 4–7) is checked to be 0 per the spec.
//   * All pointer arithmetic is bounds-checked against `total_size` before
//     any read is attempted.

use bootabi::{
    FramebufferInfo, PIXEL_FORMAT_BGR, PIXEL_FORMAT_RGB, PIXEL_FORMAT_UNKNOWN,
};
use core::{mem, ptr};

const TAG_TYPE_END:         u32 = 0;
const TAG_TYPE_FRAMEBUFFER: u32 = 8;

#[derive(Clone, Copy)]
pub struct MultibootInfo {
    base:       usize,
    total_size: usize,
}

/// Tag header common to every Multiboot2 tag.
#[repr(C)]
struct TagHeader {
    tag_type: u32,
    size:     u32,
}

impl MultibootInfo {
    /// # Safety
    /// `base` must be the physical address of a valid Multiboot2 information
    /// structure as placed by a compliant bootloader.
    pub unsafe fn new(base: usize) -> Option<Self> {
        // Reject null or misaligned pointer (spec requires 8-byte alignment).
        if base == 0 || base & 7 != 0 {
            return None;
        }

        // Read total_size and reserved using read_unaligned to avoid UB.
        let total_size = ptr::read_unaligned(base as *const u32) as usize;
        // Sanity: minimum structure is the 8-byte fixed header.
        if total_size < 8 {
            return None;
        }

        // Spec §3.1: the reserved u32 at offset 4 MUST be zero.
        let reserved = ptr::read_unaligned((base + 4) as *const u32);
        if reserved != 0 {
            return None;
        }

        Some(Self { base, total_size })
    }

    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Find and parse the framebuffer tag (type 8) in the tag list.
    pub fn framebuffer(&self) -> Option<FramebufferInfo> {
        let mut offset = 8usize; // tags start after the 8-byte fixed header

        while offset + mem::size_of::<TagHeader>() <= self.total_size {
            let tag_ptr = (self.base + offset) as *const u8;

            // Read the tag header with unaligned access.
            let header: TagHeader =
                unsafe { ptr::read_unaligned(tag_ptr as *const TagHeader) };
            let tag_size = header.size as usize;

            if header.tag_type == TAG_TYPE_END {
                break;
            }

            // Reject malformed tags.
            if tag_size < mem::size_of::<TagHeader>()
                || offset.checked_add(tag_size).map_or(true, |e| e > self.total_size)
            {
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
    // The Multiboot2 framebuffer tag layout (type 8):
    //   [0..4)   u32 type
    //   [4..8)   u32 size
    //   [8..16)  u64 framebuffer_addr
    //   [16..20) u32 pitch
    //   [20..24) u32 width
    //   [24..28) u32 height
    //   [28)     u8  bpp
    //   [29)     u8  framebuffer_type
    //   [30..32) u16 reserved
    //   [32..38) optional RGB channel descriptors (when framebuffer_type == 1)
    if tag_size < 32 {
        return None;
    }

    // All multi-byte reads use read_unaligned to avoid UB when the pointer is
    // not naturally aligned (the only guarantee is 8-byte alignment of the tag).
    let address     = unsafe { ptr::read_unaligned(tag_ptr.add(8)  as *const u64) };
    let pitch       = unsafe { ptr::read_unaligned(tag_ptr.add(16) as *const u32) };
    let width       = unsafe { ptr::read_unaligned(tag_ptr.add(20) as *const u32) };
    let height      = unsafe { ptr::read_unaligned(tag_ptr.add(24) as *const u32) };
    let bpp         = unsafe { ptr::read(tag_ptr.add(28)) }; // u8, alignment irrelevant
    let buffer_type = unsafe { ptr::read(tag_ptr.add(29)) };

    // Reasonable sanity checks.
    if address == 0 || width == 0 || height == 0 {
        return None;
    }

    // Default channel layout for the common 32bpp XRGB mode used by QEMU/GRUB.
    let mut red_field_position   = 16u8;
    let mut red_mask_size        =  8u8;
    let mut green_field_position =  8u8;
    let mut green_mask_size      =  8u8;
    let mut blue_field_position  =  0u8;
    let mut blue_mask_size       =  8u8;

    // For direct-colour framebuffers (type 1), Multiboot2 provides the exact
    // channel bit-field positions and sizes.
    if buffer_type == 1 && tag_size >= 38 {
        red_field_position   = unsafe { ptr::read(tag_ptr.add(32)) };
        red_mask_size        = unsafe { ptr::read(tag_ptr.add(33)) };
        green_field_position = unsafe { ptr::read(tag_ptr.add(34)) };
        green_mask_size      = unsafe { ptr::read(tag_ptr.add(35)) };
        blue_field_position  = unsafe { ptr::read(tag_ptr.add(36)) };
        blue_mask_size       = unsafe { ptr::read(tag_ptr.add(37)) };
    }

    Some(FramebufferInfo {
        address,
        size: u64::from(pitch) * u64::from(height),
        pitch,
        width,
        height,
        bpp,
        pixel_format: if buffer_type == 1 {
            PIXEL_FORMAT_BGR
        } else if buffer_type == 0 {
            PIXEL_FORMAT_RGB
        } else {
            PIXEL_FORMAT_UNKNOWN
        },
        red_field_position,
        red_mask_size,
        green_field_position,
        green_mask_size,
        blue_field_position,
        blue_mask_size,
        reserved: 0,
    })
}

#[inline]
fn align_up_8(value: usize) -> usize {
    (value + 7) & !7
}
