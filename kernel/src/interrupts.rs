// kernel/src/interrupts.rs
//
// Interrupt Descriptor Table (IDT) setup.
//
// Security notes:
//   * The double-fault handler is wired to IST slot 0 (via gdt::DOUBLE_FAULT_IST_INDEX),
//     ensuring it always runs on a known-good, separately allocated stack even if the
//     kernel stack has overflowed or been corrupted.
//   * Every handler emits a structured log entry before halting so that serial
//     output captures the fault context for post-mortem analysis.
//   * CR2 is read atomically inside the page-fault handler before any other code
//     can overwrite it.

use lazy_static::lazy_static;
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};
use log::{error, info};

use crate::gdt;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Fault handlers
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);

        // Double-fault MUST use the dedicated IST stack (IST slot 0) so it
        // survives a kernel-stack overflow.
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_handler);
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_fp_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_fp_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);

        idt
    };
}

/// Load the IDT.  Must be called AFTER `gdt::init()` so the TSS is active.
pub fn init() {
    IDT.load();
}

/// Enable hardware interrupts (STI).  Called once IRQ handlers are installed.
#[allow(dead_code)]
pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

/// Disable hardware interrupts (CLI).  Use sparingly; keep critical sections short.
#[allow(dead_code)]
pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

/// Spin on HLT, keeping the CPU in a low-power state while permanently halted.
#[inline(always)]
fn halt_cpu() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// ---------------------------------------------------------------------------
// Exception handlers
// ---------------------------------------------------------------------------

extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    error!("[EXCEPTION] Divide Error\n{:#?}", frame);
    halt_cpu()
}

extern "x86-interrupt" fn debug_handler(frame: InterruptStackFrame) {
    info!("[EXCEPTION] Debug\n{:#?}", frame);
}

extern "x86-interrupt" fn nmi_handler(frame: InterruptStackFrame) {
    error!("[EXCEPTION] Non-Maskable Interrupt\n{:#?}", frame);
    halt_cpu()
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    info!("[EXCEPTION] Breakpoint\n{:#?}", frame);
    // Breakpoints are resumable; do not halt.
}

extern "x86-interrupt" fn overflow_handler(frame: InterruptStackFrame) {
    error!("[EXCEPTION] Overflow\n{:#?}", frame);
    halt_cpu()
}

extern "x86-interrupt" fn bound_range_handler(frame: InterruptStackFrame) {
    error!("[EXCEPTION] Bound Range Exceeded\n{:#?}", frame);
    halt_cpu()
}

extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    error!("[EXCEPTION] Invalid Opcode (UD2)\n{:#?}", frame);
    halt_cpu()
}

extern "x86-interrupt" fn device_not_available_handler(frame: InterruptStackFrame) {
    error!("[EXCEPTION] Device Not Available (FPU)\n{:#?}", frame);
    halt_cpu()
}

/// Double-fault handler — runs on the dedicated IST stack (IST slot 0).
/// A double fault means the CPU could not deliver the original exception, which
/// typically indicates a corrupted or overflowed kernel stack.
extern "x86-interrupt" fn double_fault_handler(
    frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    error!("[EXCEPTION] DOUBLE FAULT — kernel stack likely corrupted\n{:#?}", frame);
    halt_cpu()
}

extern "x86-interrupt" fn invalid_tss_handler(frame: InterruptStackFrame, code: u64) {
    error!("[EXCEPTION] Invalid TSS (selector={:#x})\n{:#?}", code, frame);
    halt_cpu()
}

extern "x86-interrupt" fn segment_not_present_handler(frame: InterruptStackFrame, code: u64) {
    error!("[EXCEPTION] Segment Not Present (selector={:#x})\n{:#?}", code, frame);
    halt_cpu()
}

extern "x86-interrupt" fn stack_segment_handler(frame: InterruptStackFrame, code: u64) {
    error!("[EXCEPTION] Stack Segment Fault (selector={:#x})\n{:#?}", code, frame);
    halt_cpu()
}

extern "x86-interrupt" fn gpf_handler(frame: InterruptStackFrame, code: u64) {
    error!("[EXCEPTION] General Protection Fault (code={:#x})\n{:#?}", code, frame);
    halt_cpu()
}

extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    code: PageFaultErrorCode,
) {
    // Read CR2 immediately — it holds the faulting virtual address and will be
    // overwritten if any subsequent memory access triggers another fault.
    let fault_addr = x86_64::registers::control::Cr2::read().as_u64();
    error!(
        "[EXCEPTION] Page Fault at {:#018x} (flags={:?})\n{:#?}",
        fault_addr, code, frame
    );
    halt_cpu()
}

extern "x86-interrupt" fn x87_fp_handler(frame: InterruptStackFrame) {
    error!("[EXCEPTION] x87 Floating-Point Error\n{:#?}", frame);
    halt_cpu()
}

extern "x86-interrupt" fn alignment_check_handler(frame: InterruptStackFrame, code: u64) {
    error!("[EXCEPTION] Alignment Check (code={:#x})\n{:#?}", code, frame);
    halt_cpu()
}

extern "x86-interrupt" fn machine_check_handler(frame: InterruptStackFrame) -> ! {
    error!("[EXCEPTION] Machine Check — hardware error, cannot continue");
    let _ = frame;
    halt_cpu()
}

extern "x86-interrupt" fn simd_fp_handler(frame: InterruptStackFrame) {
    error!("[EXCEPTION] SIMD Floating-Point Exception\n{:#?}", frame);
    halt_cpu()
}

extern "x86-interrupt" fn virtualization_handler(frame: InterruptStackFrame) {
    error!("[EXCEPTION] Virtualization Exception\n{:#?}", frame);
    halt_cpu()
}
