#![allow(unused)]

mod tss;

use super::Mutex;
use crate::memory::MEMORY_MANAGER;
use crate::utils::*;
use core::arch::asm;
use tss::*;

static GDT: Mutex<Gdt> = Mutex::new(Gdt::new());
pub const KERNEL_CODE_SEGMENT_INDEX: usize = 1;
pub const KERNEL_DATA_SEGMENT_INDEX: usize = 2;
pub const USER_CODE_SEGMENT_INDEX: usize = 3;
pub const USER_DATA_SEGMENT_INDEX: usize = 4;
pub const TSS_SEGMENT_INDEX: usize = 5;

pub const KERNEL_CODE_SEGMENT_SELECTOR: usize = KERNEL_CODE_SEGMENT_INDEX << 3;
pub const KERNEL_DATA_SEGMENT_SELECTOR: usize = KERNEL_DATA_SEGMENT_INDEX << 3;
pub const USER_CODE_SEGMENT_SELECTOR: usize =
    (USER_CODE_SEGMENT_INDEX << 3) | PrivilegeLevel::Ring3 as usize;
pub const USER_DATA_SEGMENT_SELECTOR: usize =
    (USER_DATA_SEGMENT_INDEX << 3) | PrivilegeLevel::Ring3 as usize;
pub const TSS_SEGMENT_SELECTOR: usize = TSS_SEGMENT_INDEX << 3;

pub fn init() -> Result<(), ()> {
    // Setup segments
    {
        let mut gdt = GDT.lock();
        // Kernel code segment
        gdt.0[KERNEL_CODE_SEGMENT_INDEX].set_base(0);
        gdt.0[KERNEL_CODE_SEGMENT_INDEX].set_limit(0xfffff);
        gdt.0[KERNEL_CODE_SEGMENT_INDEX].set_dpl(PrivilegeLevel::Ring0);
        gdt.0[KERNEL_CODE_SEGMENT_INDEX].set_access(AccessOffset::Present, true);
        gdt.0[KERNEL_CODE_SEGMENT_INDEX].set_access(AccessOffset::DescriptorType, true);
        gdt.0[KERNEL_CODE_SEGMENT_INDEX].set_access(AccessOffset::Executable, true);
        gdt.0[KERNEL_CODE_SEGMENT_INDEX].set_access(AccessOffset::ReadableOrWritable, true);
        gdt.0[KERNEL_CODE_SEGMENT_INDEX].set_flag(FlagsOffset::Long, true);
        gdt.0[KERNEL_CODE_SEGMENT_INDEX].set_flag(FlagsOffset::Granularity, true);

        // Kernel data segment
        gdt.0[KERNEL_DATA_SEGMENT_INDEX].set_base(0);
        gdt.0[KERNEL_DATA_SEGMENT_INDEX].set_limit(0xfffff);
        gdt.0[KERNEL_DATA_SEGMENT_INDEX].set_dpl(PrivilegeLevel::Ring0);
        gdt.0[KERNEL_DATA_SEGMENT_INDEX].set_access(AccessOffset::Present, true);
        gdt.0[KERNEL_DATA_SEGMENT_INDEX].set_access(AccessOffset::DescriptorType, true);
        gdt.0[KERNEL_DATA_SEGMENT_INDEX].set_access(AccessOffset::Executable, false);
        gdt.0[KERNEL_DATA_SEGMENT_INDEX].set_access(AccessOffset::ReadableOrWritable, true);
        gdt.0[KERNEL_DATA_SEGMENT_INDEX].set_flag(FlagsOffset::Size, true);
        gdt.0[KERNEL_DATA_SEGMENT_INDEX].set_flag(FlagsOffset::Granularity, true);

        // User code segment
        gdt.0[USER_CODE_SEGMENT_INDEX].set_base(0);
        gdt.0[USER_CODE_SEGMENT_INDEX].set_limit(0xfffff);
        gdt.0[USER_CODE_SEGMENT_INDEX].set_dpl(PrivilegeLevel::Ring3);
        gdt.0[USER_CODE_SEGMENT_INDEX].set_access(AccessOffset::Present, true);
        gdt.0[USER_CODE_SEGMENT_INDEX].set_access(AccessOffset::DescriptorType, true);
        gdt.0[USER_CODE_SEGMENT_INDEX].set_access(AccessOffset::Executable, true);
        gdt.0[USER_CODE_SEGMENT_INDEX].set_access(AccessOffset::ReadableOrWritable, true);
        gdt.0[USER_CODE_SEGMENT_INDEX].set_flag(FlagsOffset::Long, true);
        gdt.0[USER_CODE_SEGMENT_INDEX].set_flag(FlagsOffset::Granularity, true);

        // User data segment
        gdt.0[USER_DATA_SEGMENT_INDEX].set_base(0);
        gdt.0[USER_DATA_SEGMENT_INDEX].set_limit(0xfffff);
        gdt.0[USER_DATA_SEGMENT_INDEX].set_dpl(PrivilegeLevel::Ring3);
        gdt.0[USER_DATA_SEGMENT_INDEX].set_access(AccessOffset::Present, true);
        gdt.0[USER_DATA_SEGMENT_INDEX].set_access(AccessOffset::DescriptorType, true);
        gdt.0[USER_DATA_SEGMENT_INDEX].set_access(AccessOffset::Executable, false);
        gdt.0[USER_DATA_SEGMENT_INDEX].set_access(AccessOffset::ReadableOrWritable, true);
        gdt.0[USER_DATA_SEGMENT_INDEX].set_flag(FlagsOffset::Size, true);
        gdt.0[USER_DATA_SEGMENT_INDEX].set_flag(FlagsOffset::Granularity, true);

        // TSS segment
        TSS.lock().privilege_stacks[0] = MEMORY_MANAGER.lock().physical_map.alloc_frame();
        clear_page(TSS.lock().privilege_stacks[0]);
        TSS.lock().interrupt_stacks[0] = MEMORY_MANAGER.lock().physical_map.alloc_frame();
        clear_page(TSS.lock().interrupt_stacks[0]);

        let mut tss_descriptor = SystemSegmentDescriptor::new_tss_segment(&TSS.lock());
        gdt.0[TSS_SEGMENT_INDEX].0 = tss_descriptor.0;
        gdt.0[TSS_SEGMENT_INDEX + 1].0 = tss_descriptor.1;
    }

    // Load GDT descriptor
    let descriptor = GdtDescriptor::new(&GDT.lock());
    let o = descriptor.offset;
    descriptor.load();

    Ok(())
}

#[repr(C, packed)]
struct GdtDescriptor {
    size: u16,
    offset: u64,
}

impl GdtDescriptor {
    fn new(gdt: &Gdt) -> GdtDescriptor {
        GdtDescriptor {
            size: size_of::<Gdt>() as u16 - 1,
            offset: gdt as *const Gdt as u64,
        }
    }

    fn load(&self) {
        unsafe {
            // Load GDT
            asm!("lgdt [{register}]", register = in(reg) self);

            // Reload CS
            asm!(
                "push {sel}",
                "lea {tmp}, [2f + rip]",
                "push {tmp}",
                "retfq",
                "2:",
                sel = in(reg) KERNEL_CODE_SEGMENT_SELECTOR as u64,
                tmp = lateout(reg) _,
                options(preserves_flags),
            );

            // Reload data segments
            asm!(
                "mov ds, ax",
                "mov es, ax",
                "mov fs, ax",
                "mov gs, ax",
                "mov ss, ax",
                in("ax") KERNEL_DATA_SEGMENT_SELECTOR as u16,
            );

            // Load TSS
            asm!("ltr ax", in("ax") TSS_SEGMENT_SELECTOR as u16);
        }
    }
}

#[repr(C)]
struct Gdt([GdtEntry; 7]);

impl Gdt {
    const fn new() -> Gdt {
        Gdt([GdtEntry::new(); 7])
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct GdtEntry(u64);

impl GdtEntry {
    const fn new() -> GdtEntry {
        GdtEntry(0)
    }

    // Getters
    fn get_limit(&self) -> u32 {
        ((self.0 & 0xffff) | ((self.0 >> (48 - 16)) & 0xf_0000)) as u32
    }

    fn get_base(&self) -> u64 {
        ((self.0 >> 16) & 0xffffff) | ((self.0 >> (56 - 24)) & 0xff00_0000)
    }

    fn get_accesses(&self) -> u8 {
        ((self.0 >> 40) & 0xff) as u8
    }

    fn get_access(&self, offset: AccessOffset) -> bool {
        (self.get_accesses() >> offset as u32) & 1 != 0
    }

    fn get_flags(&self) -> u8 {
        ((self.0 >> 52) & 0xf) as u8
    }

    fn get_flag(&self, offset: FlagsOffset) -> bool {
        (self.get_flags() >> offset as u32) & 1 != 0
    }

    fn get_dpl(&self) -> PrivilegeLevel {
        PrivilegeLevel::new(((self.0 >> 45) & 0b11) as u32)
    }

    // Setters
    fn set_limit(&mut self, limit: u32) {
        self.0 &= !0xf_0000_0000_ffff;
        self.0 |= (limit as u64 & 0xffff) | (((limit as u64 >> 16) & 0xf) << 48);
    }

    fn set_base(&mut self, base: u64) {
        self.0 &= !0xff00_00ff_ffff_0000;

        self.0 |= ((base & 0xff_ffff) << 16) | ((base & 0xff00_0000) << 32);
    }

    fn set_accesses(&mut self, access_byte: u8) {
        self.0 &= !0xff00_0000_0000;
        self.0 |= (access_byte as u64) << 40;
    }

    fn set_access(&mut self, offset: AccessOffset, value: bool) {
        self.0 &= !(1 << (40 + offset as u32));
        self.0 |= (value as u64) << (40 + offset as u32);
    }

    fn set_flags(&mut self, flags: u8) {
        self.0 &= !0xf0_0000_0000_0000;
        self.0 |= (flags as u64 & 0xf) << 52;
    }

    fn set_flag(&mut self, offset: FlagsOffset, value: bool) {
        self.0 &= !(1 << (52 + offset as u32));
        self.0 |= (value as u64) << (52 + offset as u32);
    }

    fn set_dpl(&mut self, privilege: PrivilegeLevel) {
        self.0 &= !0x6000_0000_0000;
        self.0 |= (privilege as u64) << 45;
    }
}

pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

impl PrivilegeLevel {
    pub fn new(ring: u32) -> PrivilegeLevel {
        match ring {
            0 => Self::Ring0,
            1 => Self::Ring1,
            2 => Self::Ring2,
            3 => Self::Ring3,
            _ => panic!("Invalid ring value"),
        }
    }
}

#[derive(Clone, Copy)]
enum AccessOffset {
    Accessed = 0,
    ReadableOrWritable = 1,
    DirectionOrConforming = 2,
    Executable = 3,
    DescriptorType = 4,
    Present = 7,
}

#[derive(Clone, Copy)]
enum FlagsOffset {
    Long = 1,
    Size = 2,
    Granularity = 3,
}
