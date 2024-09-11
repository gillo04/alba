use core::fmt::Write;

use super::InterruptStackFrame;
use crate::pic8259::*;
use crate::process::*;
use crate::stdin::*;
use crate::stdout::*;
use crate::utils::*;

pub extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    // print!(".");
    end_of_interrupt(0);
}

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    let scancode = inb(0x60);
    unsafe { STDIN.force_unlock() };
    if scancode & 128 == 0 {
        STDIN.lock().pressed_scancodes[scancode as usize] = true;
        STDIN.lock().keyboard_int = Some(scancode);
    } else {
        STDIN.lock().pressed_scancodes[(scancode & !128) as usize] = false;
        STDIN.lock().keyboard_int = None;
    }

    end_of_interrupt(1);
}

pub extern "x86-interrupt" fn print_interrupt(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    // Print
    let ptr = ctx.rax as *const u8;
    let len = ctx.rcx as usize;
    unsafe {
        let string = core::str::from_raw_parts(ptr, len);
        STDOUT.lock().write_str(string);
    }
}
