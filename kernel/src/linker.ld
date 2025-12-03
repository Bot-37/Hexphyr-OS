/* Kernel linker script */
OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

KERNEL_BASE = 0x100000;

SECTIONS {
    . = KERNEL_BASE;
    
    .text : ALIGN(4K) {
        *(.text.multiboot)
        *(.text*)
    }
    
    .rodata : ALIGN(4K) {
        *(.rodata*)
    }
    
    .data : ALIGN(4K) {
        *(.data*)
    }
    
    .bss : ALIGN(4K) {
        *(COMMON)
        *(.bss*)
    }
    
    /DISCARD/ : {
        *(.eh_frame)
        *(.note*)
        *(.comment*)
    }
    
    /* Debugging symbols */
    .stab : ALIGN(4K) {
        *(.stab*)
    }
    
    .stabstr : ALIGN(4K) {
        *(.stabstr*)
    }
}