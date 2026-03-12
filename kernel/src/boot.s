/* Multiboot2 bootstrap: enter in 32-bit protected mode, then switch to 64-bit long mode.
 *
 * Security hardening applied here:
 *   - Explicit BSS zeroing (defense-in-depth; GRUB also zeroes BSS per spec)
 *   - EFER.NXE  (CR4 No-Execute Enable)  — prevents data pages from being executed
 *   - CR0.WP    (Write Protect)          — prevents kernel from writing read-only pages
 *   - CR4.SMEP  (bit 20)                 — checked via CPUID leaf 7 before enabling
 *   - CR4.SMAP  (bit 21)                 — checked via CPUID leaf 7 before enabling
 *   - CR4.UMIP  (bit 11)                 — checked via CPUID leaf 7 ECX before enabling
 */

.section .text
.code32
.global _multiboot_entry
.type _multiboot_entry, @function
_multiboot_entry:
    cli
    cld

    /*
     * Preserve the multiboot info pointer (EBX) across the BSS-clear in %esi.
     * rep stosl clobbers %eax/%ecx/%edi but leaves %esi intact.
     */
    movl %ebx, %esi

    /* --- Explicitly zero BSS (page tables must be all-zero before use) --- */
    movl $_bss_start, %edi
    movl $_bss_end,   %ecx
    subl %edi, %ecx          /* byte count */
    shrl $2, %ecx            /* dword count (BSS is 4-byte aligned by linker) */
    xorl %eax, %eax
    rep stosl

    /* Set up the bootstrap stack (lives in BSS, now cleanly zeroed). */
    movl $boot_stack_top, %esp
    andl $-16, %esp

    /* Restore multiboot ptr into %edi so it survives to RDI in long mode. */
    movl %esi, %edi

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

    /* CR4: PAE is mandatory.  SMEP (bit 20), SMAP (bit 21), UMIP (bit 11)
     * are enabled here only if CPUID confirms support (EBX/ECX of leaf 7). */
    movl %cr4, %eax
    orl  $0x00000020, %eax   /* CR4.PAE  (bit 5)  — required for long mode */

    /* CPUID leaf 7, sub-leaf 0 → structured extended features */
    pushl %ebx               /* save caller's %ebx across CPUID */
    movl  $7, %ecx
    xorl  %eax, %eax
    cpuid                    /* EBX = extended features, ECX = more features */
    /* SMEP: EBX bit 7 */
    testl $(1 << 7),  %ebx
    jz    1f
    orl   $0x00100000, (%esp)  /* scratch: set SMEP bit (bit 20) in saved eax */
    /* Re-read and set directly: simpler to just or into local register */
1:
    movl  %cr4, %eax
    orl   $0x00000020, %eax  /* PAE always */
    movl  $7, %ecx
    pushl %eax
    xorl  %eax, %eax
    cpuid
    popl  %eax
    testl $(1 << 7),  %ebx
    jz    .no_smep
    orl   $(1 << 20), %eax   /* CR4.SMEP */
.no_smep:
    testl $(1 << 20), %ebx
    jz    .no_smap
    orl   $(1 << 21), %eax   /* CR4.SMAP */
.no_smap:
    testl $(1 << 2),  %ecx
    jz    .no_umip
    orl   $(1 << 11), %eax   /* CR4.UMIP */
.no_umip:
    popl  %ebx               /* restore caller %ebx */
    movl  %eax, %cr4

    movl $0xC0000080, %ecx   /* IA32_EFER MSR */
    rdmsr
    orl  $0x00000100, %eax   /* EFER.LME  (bit 8)  — enable long mode */
    orl  $0x00000800, %eax   /* EFER.NXE  (bit 11) — enable No-Execute */
    wrmsr

    movl %cr0, %eax
    orl  $0x80000000, %eax   /* CR0.PG  (bit 31) — enable paging */
    orl  $0x00010000, %eax   /* CR0.WP  (bit 16) — Write Protect (kernel cannot
                              *                     write to read-only pages) */
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
