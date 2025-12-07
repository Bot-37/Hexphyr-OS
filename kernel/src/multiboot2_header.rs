#[link_section = ".multiboot_header"]
#[no_mangle]
pub static MULTIBOOT2_HEADER: [u32; 8] = [
    0xE85250D6, // magic
0,          // architecture (protected-mode 32-bit)
24,         // total header length
-(0xE85250D6 as i32 + 0 + 24) as u32, // checksum
// End tag
0, 0, 8,
0
];
