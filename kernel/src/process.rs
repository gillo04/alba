#![allow(unused)]

use super::println;
use crate::gdt::*;
use crate::memory::*;
use crate::utils::*;
use alloc::vec::*;
use core::arch::asm;

const USER_STACK: u64 = 0x1000_0000;

pub struct Process {
    mappings: Vec<VirtualMapping>,
    pub context: Context,
}

impl Process {
    pub fn new(mappings: Vec<VirtualMapping>, entry_point: u64) -> Process {
        let mut tmp = Process {
            mappings,
            context: Context::new(USER_STACK),
        };

        let mut stack = VirtualMapping::new(USER_STACK, Vec::new());
        stack
            .frames
            .push(MEMORY_MANAGER.lock().physical_map.alloc_frame());
        clear_page(stack.frames[0]);
        tmp.mappings.push(stack);

        tmp.context.rip = entry_point;
        tmp
    }

    pub fn reenter(&self) {
        // Load memory mappings
        let plm4 = MEMORY_MANAGER.lock().get_plm4();
        for m in &self.mappings {
            plm4.map_mapping(m);
        }
        MEMORY_MANAGER.lock().set_plm4(plm4);

        // Load registers and jump
        self.context.load_regs();
        unsafe {
            asm!("iretq");
        }
    }
}

#[derive(Clone, Copy)]
pub struct Context {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,

    rflags: u64,
    rsp: u64,
    rip: u64,
}

impl Context {
    const fn new(stack: u64) -> Context {
        Context {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: stack,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,

            rflags: 0,
            rsp: stack,
            rip: 0,
        }
    }

    #[inline]
    fn capture_regs() -> Context {
        let mut ctx: *const Context;
        unsafe {
            asm!(
                "push 0",
                "push 0",
                "push 0",
                "push r15",
                "push r14",
                "push r13",
                "push r12",
                "push r11",
                "push r10",
                "push r9",
                "push r8",
                "push rbp",
                "push rdi",
                "push rsi",
                "push rdx",
                "push rcx",
                "push rbx",
                "push rax",
                "mov {r}, rsp",
                "add rsp, 8 * 18",
                r = out(reg) ctx
            );
        }

        unsafe { *ctx }
    }

    #[inline]
    fn load_regs(&self) {
        unsafe {
            asm!(
                "mov ds, ax",
                "mov es, ax",
                "mov fs, ax",
                "mov gs, ax",
                in("rax") KERNEL_DATA_SEGMENT_SELECTOR as u64,
            );

            asm!(
                "push {sel_data}",
                "push {sp}",
                "push {rflags}",
                "push {sel_code}",
                "push {entry}",
                sel_data = in(reg) KERNEL_DATA_SEGMENT_SELECTOR as u64,
                sp = in(reg) self.rsp,
                rflags = in(reg) self.rflags,
                sel_code = in(reg) KERNEL_CODE_SEGMENT_SELECTOR as u64,
                entry = in(reg) self.rip as u64,
                options(preserves_flags),
            );

            asm!(
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",
                "push {}",

                "pop rax",
                "pop rbx",
                "pop rcx",
                "pop rdx",
                "pop rsi",
                "pop rdi",
                "pop r8 ",
                "pop r9 ",
                "pop r10",
                "pop r11",
                "pop r12",
                "pop r13",
                "pop r14",
                "pop r15",
                "pop rbp",
                in(reg) self.rbp,
                in(reg) self.r15,
                in(reg) self.r14,
                in(reg) self.r13,
                in(reg) self.r12,
                in(reg) self.r11,
                in(reg) self.r10,
                in(reg) self.r9 ,
                in(reg) self.r8 ,
                in(reg) self.rsi,
                in(reg) self.rdi,
                in(reg) self.rdx,
                in(reg) self.rcx,
                in(reg) self.rbx,
                in(reg) self.rax,
            );
        }
    }
}
