/* Assembly boot stub */
.section .text.multiboot, "ax"
.global _multiboot_entry
.type _multiboot_entry, @function

_multiboot_entry:
    /* Clear direction flag */
    cld
    
    /* Set up stack */
    mov $stack_top, %rsp
    
    /* Align stack to 16 bytes */
    and $-16, %rsp
    
    /* Clear frame pointer */
    xor %rbp, %rbp
    
    /* Call Rust kernel entry (boot_info in RDI from UEFI loader) */
    call _start
    
    /* Should never return */
    cli
1:  hlt
    jmp 1b

.section .bss
.align 16
stack_bottom:
    .skip 16384  /* 16 KB kernel stack */
stack_top: