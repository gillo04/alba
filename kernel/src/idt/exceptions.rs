use super::println;
use super::InterruptStackFrame;
use core::arch::asm;

pub extern "x86-interrupt" fn division_error(_stack_frame: InterruptStackFrame) {
    panic!("Division by zero occurred");
}

pub extern "x86-interrupt" fn debug(_stack_frame: InterruptStackFrame) {
    panic!("Debug exception occurred");
}

pub extern "x86-interrupt" fn non_maskable_interrupt(_stack_frame: InterruptStackFrame) {
    panic!("Non maskable interrupt occurred");
}

pub extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    panic!("Breakpoint exception occurred");
}

pub extern "x86-interrupt" fn overflow(_stack_frame: InterruptStackFrame) {
    panic!("Overflow occurred");
}

pub extern "x86-interrupt" fn bound_range_exceeded(_stack_frame: InterruptStackFrame) {
    panic!("Bound range exceeded");
}

pub extern "x86-interrupt" fn invalid_opcode(stack_frame: InterruptStackFrame) {
    panic!(
        "Invalid opcode\n\tRIP: {:x}\n\tCS: {:x}",
        stack_frame.instruction_ptr, stack_frame.code_segment
    );
}

pub extern "x86-interrupt" fn device_not_available(_stack_frame: InterruptStackFrame) {
    panic!("Device not available");
}

pub extern "x86-interrupt" fn double_fault(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("Double fault occurred");
}

pub extern "x86-interrupt" fn coprocessor_segment_overrun(_stack_frame: InterruptStackFrame) {
    panic!("Coprocessor segment overrun");
}

pub extern "x86-interrupt" fn invalid_tss(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("Invalid TSS");
}

pub extern "x86-interrupt" fn segment_not_present(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("Segment not present");
}

pub extern "x86-interrupt" fn stack_segment_fault(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("Stack segment fault");
}

pub extern "x86-interrupt" fn general_protection_fault(
    _stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    let cr2: u64;
    unsafe { asm!("mov {}, cr2", out(reg) cr2) };
    panic!(
        "General protection fault\n\tCR2: 0x{:x}\n\tError: 0x{:x}",
        cr2, error_code
    );
}

pub extern "x86-interrupt" fn page_fault(stack_frame: InterruptStackFrame, error_code: u64) {
    let cr2: u64;
    unsafe { asm!("mov {}, cr2", out(reg) cr2) };
    panic!(
        "Page fault\n\tRIP: 0x{:x}\n\tCR2: 0x{:x}\n\tError: 0x{:x}",
        stack_frame.instruction_ptr, cr2, error_code
    );
}

pub extern "x86-interrupt" fn x87_floating_point_exception(_stack_frame: InterruptStackFrame) {
    panic!("X87 floating point exception occurred");
}

pub extern "x86-interrupt" fn alignment_check(_stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("Alignment check exception occurred");
}

pub extern "x86-interrupt" fn machine_check(_stack_frame: InterruptStackFrame) {
    panic!("X87 floating point exception occurred");
}

pub extern "x86-interrupt" fn simd_floating_point_exception(_stack_frame: InterruptStackFrame) {
    panic!("Simd floating point exception occurred");
}

pub extern "x86-interrupt" fn virtualization_exception(_stack_frame: InterruptStackFrame) {
    panic!("Virtualization exception occurred");
}

pub extern "x86-interrupt" fn control_protection_exception(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("Control protection exception occurred");
}

pub extern "x86-interrupt" fn hypervisor_injection_exception(_stack_frame: InterruptStackFrame) {
    panic!("Hypervisor injection exception occurred");
}

pub extern "x86-interrupt" fn vmm_communication_exception(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("Vmm communication exception occurred");
}

pub extern "x86-interrupt" fn security_exception(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("Security exception occurred");
}
