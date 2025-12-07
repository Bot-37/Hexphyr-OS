use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use log::error;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.general_protection_fault.set_handler_fn(gpf_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn divide_error_handler(stack: InterruptStackFrame) {
    error!("Divide error {:?}", stack);
    loop {}
}

extern "x86-interrupt" fn breakpoint_handler(stack: InterruptStackFrame) {
    log::info!("Breakpoint {:?}", stack);
}

extern "x86-interrupt" fn double_fault_handler(
    stack: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    error!("Double fault {:?}", stack);
    loop {}
}

extern "x86-interrupt" fn gpf_handler(
    stack: InterruptStackFrame,
    code: u64
) {
    error!("GPF {:?}, code {}", stack, code);
    loop {}
}

extern "x86-interrupt" fn page_fault_handler(
    stack: InterruptStackFrame,
    code: x86_64::structures::idt::PageFaultErrorCode
) {
    error!("PAGE FAULT {:?}, {:?}", stack, code);
    loop {}
}
pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
}
