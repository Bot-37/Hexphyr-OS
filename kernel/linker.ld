ENTRY(_multiboot_entry)

SECTIONS
{
    . = 1M;

    .multiboot_header :
    {
        KEEP(*(.multiboot_header))
    }

    .text :
    {
        *(.text*)
    }

    .rodata :
    {
        *(.rodata*)
    }

    .data :
    {
        *(.data*)
    }

    .bss :
    {
        _bss_start = .;
        *(COMMON)
        *(.bss*)
        . = ALIGN(4);       /* Required by rep-stosl (dword stride) in boot.s */
        _bss_end = .;
    }

    /* Strip sections that are not needed at runtime and bloat the image. */
    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.note.GNU-stack)   /* Mark stack non-executable at link time */
    }
}

