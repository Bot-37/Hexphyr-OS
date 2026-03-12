/* Multiboot2 bootstrap: enter in 32-bit protected mode, then switch to 64-bit long mode. */

.section .text
.code32
.global _multiboot_entry
.type _multiboot_entry, @function
_multiboot_entry:
    cli
    cld

    movl $boot_stack_top, %esp
    andl $-16, %esp

    /* GRUB puts the multiboot info pointer in EBX. Keep it for Rust _start(RDI). */
    movl %ebx, %edi

    call setup_page_tables
    call enable_long_mode

    lgdt gdt64_descriptor
    ljmp $0x08, $long_mode_entry

setup_page_tables:
    /* PML4[0] -> PDPT */
    movl $page_table_l3, %eax
    orl $0x3, %eax
    movl %eax, page_table_l4
    movl $0, page_table_l4 + 4

    /* PDPT[0] -> PD */
    movl $page_table_l2, %eax
    orl $0x3, %eax
    movl %eax, page_table_l3
    movl $0, page_table_l3 + 4

    /* Identity-map first 1 GiB using 2 MiB pages. */
    xorl %ecx, %ecx
1:
    movl %ecx, %eax
    shll $21, %eax
    orl $0x83, %eax
    movl %eax, page_table_l2(,%ecx,8)
    movl $0, page_table_l2 + 4(,%ecx,8)
    incl %ecx
    cmpl $512, %ecx
    jne 1b

    ret

enable_long_mode:
    movl $page_table_l4, %eax
    movl %eax, %cr3

    movl %cr4, %eax
    orl $0x20, %eax       /* CR4.PAE */
    movl %eax, %cr4

    movl $0xC0000080, %ecx /* EFER MSR */
    rdmsr
    orl $0x100, %eax      /* EFER.LME */
    wrmsr

    movl %cr0, %eax
    orl $0x80000000, %eax /* CR0.PG */
    movl %eax, %cr0

    ret

.code64
long_mode_entry:
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    mov %ax, %fs
    mov %ax, %gs

    movabs $boot_stack_top, %rsp
    andq $-16, %rsp
    xorq %rbp, %rbp

    /* Zero-extend the multiboot pointer and pass as first arg to Rust _start. */
    movl %edi, %edi
    call _start

2:
    hlt
    jmp 2b

.align 8
gdt64:
    .quad 0x0000000000000000
    .quad 0x00af9a000000ffff /* code segment */
    .quad 0x00af92000000ffff /* data segment */
gdt64_end:

gdt64_descriptor:
    .word gdt64_end - gdt64 - 1
    .long gdt64

.section .bss
.align 16
boot_stack_bottom:
    .skip 65536
boot_stack_top:

.align 4096
page_table_l4:
    .skip 4096

.align 4096
page_table_l3:
    .skip 4096

.align 4096
page_table_l2:
    .skip 4096
