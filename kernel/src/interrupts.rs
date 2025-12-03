use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // Set up exception handlers
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        
        idt
    };
}

pub fn init() {
    IDT.load();
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    log::error!("Divide error at {:?}", stack_frame.instruction_pointer);
    loop {}
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log::info!("Breakpoint at {:?}", stack_frame.instruction_pointer);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    log::error!("Double fault at {:?}", stack_frame.instruction_pointer);
    loop {}
}