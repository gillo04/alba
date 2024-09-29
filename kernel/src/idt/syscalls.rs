#![allow(unused)]

use super::InterruptStackFrame;
use super::{isr::*, println};
use crate::process::*;

#[inline(always)]
pub fn enter_syscall(stack_frame: InterruptStackFrame) -> (usize, Context) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    let mut current_process = PROCESS_LIST.lock().current_process;
    PROCESS_LIST.lock().processes[current_process].context = ctx.clone();

    (current_process, ctx)
}

#[inline(always)]
pub fn exit_syscall(current_process: usize) {
    PROCESS_LIST.lock().processes[current_process].reenter();
}

pub extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
    let (current_process, ctx) = enter_syscall(stack_frame);

    match ctx.rax {
        0x10 => print(ctx),
        0x11 => put_screen_buffer(ctx),
        0x12 => get_screen_size(current_process, ctx),

        0x20 => get_key(current_process, ctx),
        0x21 => get_mouse(current_process, ctx),

        0x30 => load_file(current_process, ctx),

        0x40 => alloc_pages(current_process, ctx),
        0x41 => get_shared_page(current_process, ctx),

        0x50 => get_milliseconds_since_startup(current_process, ctx),
        0x60 => exec(current_process, ctx),
        _ => panic!("Call to unknown syscall"),
    }

    exit_syscall(current_process);
}
