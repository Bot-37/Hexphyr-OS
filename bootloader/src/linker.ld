/* EFI application linker script */
OUTPUT_FORMAT(elf64-x86-64)
ENTRY(efi_main)

SECTIONS {
    . = 0;
    
    .text : {
        *(.text*)
        *(.rodata*)
    }
    
    .data : {
        *(.data*)
    }
    
    .reloc : {
        *(.reloc*)
    }
    
    . = ALIGN(4096);
    
    .bss : {
        *(COMMON)
        *(.bss*)
    }
    
    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
    }
}