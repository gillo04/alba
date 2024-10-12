use super::super::print;
use super::syscalls::*;
use super::*;
use crate::fat32::*;
use crate::ipc;
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
use alloc::string::*;
use core::arch::*;
use core::fmt::Write;
use core::str::FromStr;

pub extern "x86-interrupt" fn timer_handler(stack_frame: InterruptStackFrame) {
    let mut ctx = Context::capture_regs();
    ctx.rsp = stack_frame.stack_ptr;
    ctx.rip = stack_frame.instruction_ptr;
    ctx.rflags = stack_frame.r_flags;

    if PROCESS_LIST.lock().processes.len() == 0 {
        panic!("Empty process list");
    }

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
            MOUSE_POS.lock().2 = buttons & 1 != 0;
            MOUSE_POS.lock().3 = buttons & (1 << 1) != 0;
        }
    }
    end_of_interrupt(12);
}

pub fn print(ctx: Context) {
    let ptr = ctx.rcx as *const u8;
    let len = ctx.rdx as usize;
    unsafe {
        let string = core::str::from_raw_parts(ptr, len);
        STDOUT.lock().write_str(string);
    }
}

pub fn put_screen_buffer(ctx: Context) {
    let buffer = ctx.rcx as *const u32;
    let mut x = ctx.rdx;
    let mut y = ctx.r8;
    let mut w = ctx.r9;
    let mut h = ctx.r10;
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
        }
    }
}

pub fn get_screen_size(current_process: usize, ctx: Context) {
    PROCESS_LIST.lock().processes[current_process].context.rcx = STDOUT.lock().frame_buffer.width;
    PROCESS_LIST.lock().processes[current_process].context.rdx = STDOUT.lock().frame_buffer.height;
}

pub fn load_file(current_process: usize, ctx: Context) {
    let path = {
        let ptr = PROCESS_LIST.lock().processes[current_process].context.rcx;
        let len = PROCESS_LIST.lock().processes[current_process].context.rdx;
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

    PROCESS_LIST.lock().processes[current_process].context.r8 = ptr;
    PROCESS_LIST.lock().processes[current_process].context.r9 = size;
}

pub fn alloc_pages(current_process: usize, ctx: Context) {
    let page_count = PROCESS_LIST.lock().processes[current_process].context.rcx;

    let mapping = KERNEL_VALLOCATOR.lock().alloc_pages(page_count);
    let vaddr = mapping.vaddr;
    PROCESS_LIST.lock().processes[current_process]
        .mappings
        .push(mapping);

    PROCESS_LIST.lock().processes[current_process].context.rdx = vaddr;
}

pub fn get_mouse(current_process: usize, ctx: Context) {
    PROCESS_LIST.lock().processes[current_process].context.rcx = MOUSE_POS.lock().0;
    PROCESS_LIST.lock().processes[current_process].context.rdx = MOUSE_POS.lock().1;
    PROCESS_LIST.lock().processes[current_process].context.r8 = MOUSE_POS.lock().2 as u64;
    PROCESS_LIST.lock().processes[current_process].context.r9 = MOUSE_POS.lock().3 as u64;
}

pub fn get_key(current_process: usize, ctx: Context) {
    PROCESS_LIST.lock().processes[current_process].context.rcx =
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

    PROCESS_LIST.lock().processes[current_process].context.rdx = c as u64;
    PROCESS_LIST.lock().processes[current_process].context.r8 = sc as u64;
    STDIN.lock().keyboard_int = None;
}

pub fn exec(current_process: usize, ctx: Context) {
    let ptr = ctx.rcx as *const u8;
    let len = ctx.rdx as usize;
    let string = unsafe { core::str::from_raw_parts(ptr, len) };
    let fat = FAT32.lock();
    let file = fat.as_ref().unwrap().read_file(string);
    match file {
        Ok(f) => {
            PROCESS_LIST.lock().processes[current_process].context.r8 = 1;
            let proc = crate::elf::ElfExecutable::new(f);
            let pid = PROCESS_LIST
                .lock()
                .push_process(proc.load_all(), proc.get_entry());
            PROCESS_LIST.lock().processes[current_process].context.r9 = pid as u64;
        }
        Err(e) => {
            PROCESS_LIST.lock().processes[current_process].context.r8 = 0;
        }
    }
    unsafe {
        FAT32.force_unlock();
    }
}

// For some reason, if this function is automatically inlined, the kernel will throw a GP fault,
// even if neither syscall_handler nor this specific routine have been called...
#[inline(never)]
pub fn get_shared_page(current_process: usize, ctx: Context) {
    PROCESS_LIST.lock().processes[current_process].context.rcx = *SHARED_PAGE.lock();
}

pub fn get_milliseconds_since_startup(current_process: usize, ctx: Context) {
    PROCESS_LIST.lock().processes[current_process].context.rcx = *MILLISECONDS_SINCE_STARTUP.lock();
}

pub fn exit(current_process: usize, ctx: Context) {
    let mut pid = PROCESS_LIST.lock().processes[current_process].pid;
    PROCESS_LIST.lock().kill(pid);
}

pub fn kill(current_process: usize, ctx: Context) {
    let mut pid = ctx.rcx as u32;
    PROCESS_LIST.lock().kill(pid);
}

pub fn create_mail_box(current_process: usize, ctx: Context) {
    let mut pid = PROCESS_LIST.lock().processes[current_process].pid;
    let name = unsafe {
        let ptr = ctx.rcx;
        let len = ctx.rdx;
        core::str::from_raw_parts(ptr as *const u8, len as usize)
    };
    let name = String::from(name);

    ipc::create_mail_box(pid, name);
}

pub fn delete_mail_box(current_process: usize, ctx: Context) {
    let mut pid = PROCESS_LIST.lock().processes[current_process].pid;
    let name = unsafe {
        let ptr = ctx.rcx;
        let len = ctx.rdx;
        core::str::from_raw_parts(ptr as *const u8, len as usize)
    };
    let name = String::from(name);

    ipc::delete_mail_box(pid, name);
}

pub fn send_message(current_process: usize, ctx: Context) {
    let mut pid = PROCESS_LIST.lock().processes[current_process].pid;
    let name = unsafe {
        let ptr = ctx.rcx;
        let len = ctx.rdx;
        core::str::from_raw_parts(ptr as *const u8, len as usize)
    };
    let name = String::from(name);

    let data = unsafe {
        let ptr = ctx.r8;
        let len = ctx.r9;
        core::slice::from_raw_parts(ptr as *const u8, len as usize)
    };

    ipc::send(pid, &name, data);
}

// TODO figure out buffer size checking
pub fn try_receive_message(current_process: usize, ctx: Context) {
    let mut pid = PROCESS_LIST.lock().processes[current_process].pid;
    let name = unsafe {
        let ptr = ctx.rcx;
        let len = ctx.rdx;
        core::str::from_raw_parts(ptr as *const u8, len as usize)
    };
    let name = String::from(name);

    if let Ok(msg) = ipc::try_receive(&name) {
        if let Some(msg) = msg {
            PROCESS_LIST.lock().processes[current_process].context.r10 = 0;
            let dest = unsafe {
                let ptr = ctx.r8;
                let len = msg.data.len();
                core::slice::from_raw_parts_mut(ptr as *mut u8, len as usize)
            };
            dest.copy_from_slice(&msg.data);
        } else {
            PROCESS_LIST.lock().processes[current_process].context.r10 = 1;
        }
    } else {
        PROCESS_LIST.lock().processes[current_process].context.r10 = 1;
    }
}
