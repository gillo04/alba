use core::fmt::Write;

use super::InterruptStackFrame;
use super::{super::print, println};
use crate::fat32::*;
use crate::memory::*;
use crate::mouse::*;
use crate::pic8259::*;
use crate::pit::*;
use crate::process::*;
use crate::stdin::scancodes::*;
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

    let mut current_process = PROCESS_LIST.lock().current_process;
    if PROCESS_LIST.lock().jump_to_multitasking {
        PROCESS_LIST.lock().jump_to_multitasking = false;
    } else {
        PROCESS_LIST.lock().processes[current_process].context = ctx;
    }
    // PROCESS_LIST.lock().processes[current_process].invalidate_tlb();

    *MILLISECONDS_SINCE_STARTUP.lock() += 1;
    end_of_interrupt(0);

    // Switch task
    current_process = (current_process + 1) % PROCESS_LIST.lock().processes.len();
    PROCESS_LIST.lock().current_process = current_process;
    PROCESS_LIST.lock().processes[current_process].reenter();
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

pub extern "x86-interrupt" fn mouse_handler(_stack_frame: InterruptStackFrame) {
    while inb(0x64) & 1 != 0 {
        let buttons = inb(0x60);
        let mut x = inb(0x60) as u32;
        let mut y = inb(0x60) as u32;
        if buttons & (0b11 << 6) == 0 {
            if buttons & (1 << 4) != 0 {
                x |= 0xffffff00;
            }
            if buttons & (1 << 5) != 0 {
                y |= 0xffffff00;
            }

            let x = x as i32;
            let mut y = y as i32;
            y = -y;

            let fb = &STDOUT.lock().frame_buffer;
            let mx = i64::clamp(MOUSE_POS.lock().0 as i64 + x as i64, 0, fb.width as i64) as u64;
            let my = i64::clamp(MOUSE_POS.lock().1 as i64 + y as i64, 0, fb.height as i64) as u64;
            MOUSE_POS.lock().0 = mx;
            MOUSE_POS.lock().1 = my;
        }
    }
    end_of_interrupt(12);
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
    let base = frame_buffer.base as *mut u32;

    for i in 0..h {
        unsafe {
            core::ptr::copy_nonoverlapping(
                buffer.offset((i * w) as isize),
                base.offset(((y + i) * frame_buffer.pixels_per_scanline + x) as isize),
                w as usize,
            );
            /*(frame_buffer.base as *mut u32)
            .offset(((y + i) * frame_buffer.pixels_per_scanline + (x + j)) as isize) =
            *(buffer as *mut u32).offset((i * w + j) as isize);*/
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

pub extern "x86-interrupt" fn get_mouse_pos(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    let mut current_process = PROCESS_LIST.lock().current_process;
    PROCESS_LIST.lock().processes[current_process].context = ctx;

    // INTERRUPT CODE
    PROCESS_LIST.lock().processes[current_process].context.rax = MOUSE_POS.lock().0;
    PROCESS_LIST.lock().processes[current_process].context.rcx = MOUSE_POS.lock().1;
    // END OF INTERRUPT CODE

    PROCESS_LIST.lock().processes[current_process].reenter();
}

pub extern "x86-interrupt" fn get_key(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    let mut current_process = PROCESS_LIST.lock().current_process;
    PROCESS_LIST.lock().processes[current_process].context = ctx;

    // INTERRUPT CODE
    PROCESS_LIST.lock().processes[current_process].context.rax =
        STDIN.lock().keyboard_int.is_some() as u64;

    let int = STDIN.lock().keyboard_int;
    let mut c = '\0';
    let mut sc = 0;
    if let Some(scancode) = int {
        sc = scancode;
        // Caps lock
        if scancode == 0x3a {
            let cl = STDIN.lock().is_caps_lock_active;
            STDIN.lock().is_caps_lock_active = !cl;
        }

        if SCAN_CODE_SET1[scancode as usize] != '\0' {
            let mut char = SCAN_CODE_SET1[scancode as usize];

            let cl = STDIN.lock().is_caps_lock_active;
            let mut s = STDIN.lock().pressed_scancodes[0x2a];
            s |= STDIN.lock().pressed_scancodes[0x36];
            if !(cl ^ s) {
                if char >= 'A' && char <= 'Z' {
                    char = char.to_ascii_lowercase();
                }
            }

            if s {
                char = SHIFT_SET1[scancode as usize] as char;
            }
            c = char;
        }
    }

    PROCESS_LIST.lock().processes[current_process].context.rcx = c as u64;
    PROCESS_LIST.lock().processes[current_process].context.rdx = sc as u64;
    STDIN.lock().keyboard_int = None;
    // END OF INTERRUPT CODE

    PROCESS_LIST.lock().processes[current_process].reenter();
}

pub extern "x86-interrupt" fn exec(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    let mut current_process = PROCESS_LIST.lock().current_process;
    PROCESS_LIST.lock().processes[current_process].context = ctx;

    // INTERRUPT CODE
    let ptr = ctx.rax as *const u8;
    let len = ctx.rcx as usize;
    let string = unsafe { core::str::from_raw_parts(ptr, len) };
    let fat = FAT32.lock();
    let file = fat.as_ref().unwrap().read_file(string);
    match file {
        Ok(f) => {
            PROCESS_LIST.lock().processes[current_process].context.rdx = 1;
            let proc = crate::elf::ElfExecutable::new(f);
            let proc = Process::new(proc.load_all(), proc.get_entry());
            PROCESS_LIST.lock().processes.push(proc);
        }
        Err(e) => {
            PROCESS_LIST.lock().processes[current_process].context.rdx = 0;
        }
    }
    unsafe {
        FAT32.force_unlock();
    }
    // END OF INTERRUPT CODE
    PROCESS_LIST.lock().processes[current_process].reenter();
}
