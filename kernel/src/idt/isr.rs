use core::fmt::Write;

use super::InterruptStackFrame;
use super::{super::print, println};
use crate::fat32::*;
use crate::memory::*;
use crate::pic8259::*;
use crate::pit::*;
use crate::process::*;
use crate::stdin::*;
use crate::stdout::*;
use crate::utils::*;
use crate::Fs;
use core::arch::*;

pub extern "x86-interrupt" fn timer_handler(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    let mut current_process = 0;
    let active = PROCESS_LIST.lock().multitasking_active;
    if active {
        current_process = PROCESS_LIST.lock().current_process;
        if PROCESS_LIST.lock().jump_to_multitasking {
            PROCESS_LIST.lock().jump_to_multitasking = false;
        } else {
            PROCESS_LIST.lock().processes[current_process].context = ctx;
        }
    }

    *MILLISECONDS_SINCE_STARTUP.lock() += 1;
    end_of_interrupt(0);

    // Switch task
    if active {
        current_process = (current_process + 1) % PROCESS_LIST.lock().processes.len();
        PROCESS_LIST.lock().current_process = current_process;
        PROCESS_LIST.lock().processes[current_process].reenter();
    }
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

    // Print
    let ptr = ctx.rax as *const u8;
    let len = ctx.rcx as usize;
    unsafe {
        let string = core::str::from_raw_parts(ptr, len);
        STDOUT.lock().write_str(string);
    }
}

pub extern "x86-interrupt" fn put_screen_buffer(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();

    let buffer = ctx.rax as *const u32;
    let mut x = ctx.rcx;
    let mut y = ctx.rdx;
    let mut w = ctx.r8;
    let mut h = ctx.r9;
    let frame_buffer = STDOUT.lock().frame_buffer;

    // Bounds checking
    x = u64::min(x, frame_buffer.width);
    y = u64::min(y, frame_buffer.height);
    if x + w >= frame_buffer.width {
        w = frame_buffer.width - x;
    }
    if y + h >= frame_buffer.height {
        h = frame_buffer.height - y;
    }

    for i in 0..h {
        for j in 0..w {
            unsafe {
                *(frame_buffer.base as *mut u32)
                    .offset(((y + i) * frame_buffer.pixels_per_scanline + (x + j)) as isize) =
                    *(buffer as *mut u32).offset((i * w + j) as isize);
            }
        }
    }
}

pub extern "x86-interrupt" fn get_screen_size(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    let mut current_process = PROCESS_LIST.lock().current_process;
    PROCESS_LIST.lock().processes[current_process].context = ctx;

    // INTERRUPT CODE
    PROCESS_LIST.lock().processes[current_process].context.rax = STDOUT.lock().frame_buffer.width;
    PROCESS_LIST.lock().processes[current_process].context.rcx = STDOUT.lock().frame_buffer.height;
    // END OF INTERRUPT CODE

    PROCESS_LIST.lock().processes[current_process].reenter();
}

pub extern "x86-interrupt" fn load_file(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    let mut current_process = PROCESS_LIST.lock().current_process;
    PROCESS_LIST.lock().processes[current_process].context = ctx;

    // INTERRUPT CODE
    let path = {
        let ptr = PROCESS_LIST.lock().processes[current_process].context.rax;
        let len = PROCESS_LIST.lock().processes[current_process].context.rcx;
        unsafe { core::str::from_raw_parts(ptr as *const u8, len as usize) }
    };

    let mut ptr = 0;
    let mut size = 0;
    if let Ok(file) = FAT32.lock().as_ref().unwrap().read_file(path) {
        ptr = file.mapping.vaddr;
        size = file.size;
        PROCESS_LIST.lock().processes[current_process]
            .mappings
            .push(file.mapping);
    }

    PROCESS_LIST.lock().processes[current_process].context.rdx = ptr;
    PROCESS_LIST.lock().processes[current_process].context.r8 = size;
    // END OF INTERRUPT CODE

    PROCESS_LIST.lock().processes[current_process].reenter();
}

pub extern "x86-interrupt" fn get_milliseconds_since_startup(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    let mut current_process = PROCESS_LIST.lock().current_process;
    PROCESS_LIST.lock().processes[current_process].context = ctx;

    PROCESS_LIST.lock().processes[current_process].context.rax = *MILLISECONDS_SINCE_STARTUP.lock();
    PROCESS_LIST.lock().processes[current_process].reenter();
}

pub extern "x86-interrupt" fn alloc_pages(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    let mut current_process = PROCESS_LIST.lock().current_process;
    PROCESS_LIST.lock().processes[current_process].context = ctx;

    // INTERRUPT CODE
    let page_count = PROCESS_LIST.lock().processes[current_process].context.rax;

    let mapping = KERNEL_VALLOCATOR.lock().alloc_pages(page_count);
    let vaddr = mapping.vaddr;
    PROCESS_LIST.lock().processes[current_process]
        .mappings
        .push(mapping);

    PROCESS_LIST.lock().processes[current_process].context.rcx = vaddr;
    // END OF INTERRUPT CODE

    PROCESS_LIST.lock().processes[current_process].reenter();
}
