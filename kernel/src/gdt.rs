// kernel/src/gdt.rs
//
// Global Descriptor Table + Task State Segment (TSS).
//
// The primary reason for a TSS in a 64-bit kernel is the Interrupt Stack Table
// (IST): certain exception handlers (especially the double-fault handler) MUST
// run on a known-good, separately allocated stack, because the exception itself
// may have been caused by a corrupted or overflowed stack.  Without an IST, a
// double fault triggered by a stack overflow would immediately triple-fault,
// rebooting the machine with no diagnostic output.
//
// Security posture:
//   * A fresh GDT replaces the minimal assembly GDT from boot.s, adding the TSS
//     descriptor required for IST entries.
//   * CS/SS/DS/ES are reloaded so any hidden segment-cache state from the
//     assembly stub is flushed.

use lazy_static::lazy_static;
use x86_64::{
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};
use core::ptr::addr_of;

/// IST slot index used for the double-fault exception handler stack.
/// Valid values are 0–6 (maps to IST1–IST7 in the TSS).
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Size of the dedicated double-fault stack: 20 KiB.
/// Large enough to handle a nested page fault while printing diagnostics via
/// the serial logger.
const DOUBLE_FAULT_STACK_SIZE: usize = 20 * 1024;

/// The stack itself.  Lives in BSS and is explicitly zeroed by boot.s before
/// the BSS region is used.  The `#[repr(align(16))]` matches the x86-64 ABI
/// requirement that %rsp is 16-byte aligned on exception entry.
#[repr(align(16))]
struct AuxStack([u8; DOUBLE_FAULT_STACK_SIZE]);

static mut DOUBLE_FAULT_STACK: AuxStack = AuxStack([0; DOUBLE_FAULT_STACK_SIZE]);

struct Selectors {
    code_sel: SegmentSelector,
    data_sel: SegmentSelector,
    tss_sel:  SegmentSelector,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // Point IST slot 0 to the *top* of the double-fault stack (stack grows down).
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = unsafe {
            // Use addr_of! to obtain the stack's base address without creating
            // a reference to the mutable static (which is UB in Rust 2024).
            let stack_bottom = addr_of!(DOUBLE_FAULT_STACK.0) as u64;
            VirtAddr::new(stack_bottom + DOUBLE_FAULT_STACK_SIZE as u64)
        };
        tss
    };

    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_sel = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_sel = gdt.add_entry(Descriptor::kernel_data_segment());
        let tss_sel  = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_sel, data_sel, tss_sel })
    };
}

/// Load the production GDT, reload all segment registers, and install
/// the TSS so that the double-fault IST is active.
///
/// Must be called before `interrupts::init()`.
pub fn init() {
    use x86_64::instructions::{
        segmentation::{CS, DS, ES, SS, Segment},
        tables::load_tss,
    };

    GDT.0.load();

    // SAFETY: The selectors come from the GDT we just loaded; they are valid
    // and have the correct privilege level (0) for kernel-mode operation.
    unsafe {
        CS::set_reg(GDT.1.code_sel);
        // In 64-bit mode the SS DPL must match CPL (0); load the data descriptor.
        SS::set_reg(GDT.1.data_sel);
        // DS/ES are largely ignored in 64-bit mode but must be consistent.
        DS::set_reg(GDT.1.data_sel);
        ES::set_reg(GDT.1.data_sel);
        load_tss(GDT.1.tss_sel);
    }
}
