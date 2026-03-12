const MB2_MAGIC: u32 = 0xE85250D6;
const MB2_ARCH: u32 = 0;
const HEADER_LEN: u32 = 48;
const HEADER_CHECKSUM: u32 =
    (0u32).wrapping_sub(MB2_MAGIC.wrapping_add(MB2_ARCH).wrapping_add(HEADER_LEN));

#[repr(C, align(8))]
pub struct MultibootHeader(pub [u32; 12]);

#[used]
#[link_section = ".multiboot_header"]
#[no_mangle]
pub static MULTIBOOT2_HEADER: MultibootHeader = MultibootHeader([
    MB2_MAGIC,
    MB2_ARCH,
    HEADER_LEN,
    HEADER_CHECKSUM,
    // Framebuffer request tag: type=5, flags=0, size=20, width=1024, height=768, depth=32
    0x0000_0005,
    20,
    1024,
    768,
    32,
    0, // padding so next tag starts at 8-byte boundary
    // End tag: type=0, flags=0, size=8
    0x0000_0000,
    8,
]);
