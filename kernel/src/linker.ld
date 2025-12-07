OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

SECTIONS
{
    /* Higher-half kernel base without underscores */
    . = 0xFFFFFFFF80000000;

    .text : ALIGN(4K) {
        *(.text*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata*)
    }

    .data : ALIGN(4K) {
        *(.data*)
    }

    .bss : ALIGN(4K) {
        *(.bss*)
        *(COMMON)
    }

    /DISCARD/ : {
        *(.eh_frame*)
        *(.comment)
    }
}
