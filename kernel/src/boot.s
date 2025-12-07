/* Assembly boot stub */
.section .text
.global _multiboot_entry
.type _multiboot_entry, @function

_multiboot_entry:
    cli
    cld

    mov $stack_top, %rsp
    and $-16, %rsp

    xor %rbp, %rbp

    // GRUB loads MBI pointer into %ebx (32-bit)
    mov %rbx, %rdi

    call _start

.hang:
    hlt
    jmp .hang

.section .bss
.align 16
stack_bottom:
    .skip 16384
stack_top:
