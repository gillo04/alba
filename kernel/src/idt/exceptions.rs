use super::InterruptStackFrame;

pub extern "x86-interrupt" fn division_error(_stack_frame: InterruptStackFrame) {
    panic!("Division by zero occurred");
}

// pub extern "x86-interrupt" fn page_fault_handler(_stack_frame: InterruptStackFrame, error_code: u64)
