#![allow(unused)]

use super::println;
use super::Mutex;
use crate::gdt::*;
use crate::memory::*;
use crate::utils::*;
use alloc::vec::*;
use core::arch::asm;

pub static PROCESS_LIST: Mutex<ProcessList> = Mutex::new(ProcessList::new());

const USER_STACK_BASE: u64 = 0x1000_0000;
const USER_STACK_PAGE_COUNT: u64 = 0x1000;

pub struct ProcessList {
    pub processes: Vec<Process>,
    pub current_process: usize,
    pub jump_to_multitasking: bool,
}

impl ProcessList {
    const fn new() -> ProcessList {
        ProcessList {
            processes: Vec::new(),
            current_process: 0,
            jump_to_multitasking: false,
        }
    }
}

pub struct Process {
    pub mappings: Vec<VirtualMapping>,
    pub context: Context,
}

impl Process {
    pub fn new(mappings: Vec<VirtualMapping>, entry_point: u64) -> Process {
        let mut tmp = Process {
            mappings,
            context: Context::new(USER_STACK_BASE + USER_STACK_PAGE_COUNT * 0x1000),
        };

        let mut stack = VirtualMapping::new(USER_STACK_BASE, Vec::new());
        for i in 0..USER_STACK_PAGE_COUNT {
            stack
                .frames
                .push(MEMORY_MANAGER.lock().physical_map.alloc_frame());
            clear_page(stack.frames[0]);
        }
        tmp.mappings.push(stack);

        tmp.context.rip = entry_point;
        tmp
    }

    #[inline(always)]
    pub fn reenter(&mut self) {
        // Load memory mappings
        let plm4 = MEMORY_MANAGER.lock().get_plm4();
        for m in &self.mappings {
            plm4.map_mapping_user(m);
        }
        // Flush cr3
        MEMORY_MANAGER.lock().set_plm4(plm4);

        // Create stack guard page
        plm4.unmap(USER_STACK_BASE - 0x1000);

        // Load registers and jump
        self.context.load_regs();

        unsafe {
            PROCESS_LIST.force_unlock();
            asm!("iretq");
        }
    }

    pub fn invalidate_tlb(&self) {
        for m in &self.mappings {
            for i in 0..m.frames.len() {
                unsafe {
                    asm!(
                        "invlpg [{}]",
                        in(reg) m.vaddr + i as u64 * 0x1000
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    pub rflags: u64,
    pub rsp: u64,
    pub rip: u64,
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

            rflags: 0x0200,
            rsp: stack,
            rip: 0,
        }
    }

    #[inline]
    pub fn capture_regs() -> Context {
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
                in("rax") USER_DATA_SEGMENT_SELECTOR as u64,
            );

            asm!(
                "push {sel_data}",
                "push {sp}",
                "push {rflags}",
                "push {sel_code}",
                "push {entry}",
                // "add rsp, 5*8",
                sel_data = in(reg) USER_DATA_SEGMENT_SELECTOR as u64,
                sp = in(reg) self.rsp,
                rflags = in(reg) self.rflags,
                sel_code = in(reg) USER_CODE_SEGMENT_SELECTOR as u64,
                entry = in(reg) self.rip as u64,
                options(preserves_flags),
            );

            asm!(
                // "sub rsp, 5*8",
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
                in(reg) self.rdi,
                in(reg) self.rsi,
                in(reg) self.rdx,
                in(reg) self.rcx,
                in(reg) self.rbx,
                in(reg) self.rax,
            );
        }
    }
}
