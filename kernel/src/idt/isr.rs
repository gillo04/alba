use super::InterruptStackFrame;
use crate::pic8259::*;
use crate::print;
use crate::utils::*;

pub extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");
    end_of_interrupt(0);
}

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    let scancode = inb(0x60);

    print!("{} ", scancode);

    end_of_interrupt(1);
}
