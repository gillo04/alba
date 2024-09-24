#![allow(unused)]

mod exceptions;
mod isr;

use super::{println, Mutex};
use crate::gdt::{PrivilegeLevel, KERNEL_CODE_SEGMENT_SELECTOR};
use core::arch::asm;
use exceptions::*;
use isr::*;

static IDT: Mutex<Idt> = Mutex::new(Idt::new());

pub fn init() -> Result<(), ()> {
    // Setup interrupt service routines
    {
        let mut idt = IDT.lock();

        // Exception ISRs
        idt.0[ExceptionIndex::DivisionError as usize].set_exception_handler(division_error);
        idt.0[ExceptionIndex::Debug as usize].set_exception_handler(debug);
        idt.0[ExceptionIndex::NonMaskableInterrupt as usize]
            .set_exception_handler(non_maskable_interrupt);
        idt.0[ExceptionIndex::Breakpoint as usize].set_exception_handler(breakpoint);
        idt.0[ExceptionIndex::Overflow as usize].set_exception_handler(overflow);
        idt.0[ExceptionIndex::BoundRangeExceeded as usize]
            .set_exception_handler(bound_range_exceeded);
        idt.0[ExceptionIndex::InvalidOpcode as usize].set_exception_handler(invalid_opcode);
        idt.0[ExceptionIndex::DeviceNotAvailable as usize]
            .set_exception_handler(device_not_available);
        idt.0[ExceptionIndex::DoubleFault as usize].set_exception_handler_with_error(double_fault);
        idt.0[ExceptionIndex::CoprocessorSegmentOverrun as usize]
            .set_exception_handler(coprocessor_segment_overrun);
        idt.0[ExceptionIndex::InvalidTSS as usize].set_exception_handler_with_error(invalid_tss);
        idt.0[ExceptionIndex::SegmentNotPresent as usize]
            .set_exception_handler_with_error(segment_not_present);
        idt.0[ExceptionIndex::StackSegmentFault as usize]
            .set_exception_handler_with_error(stack_segment_fault);
        idt.0[ExceptionIndex::GeneralProtectionFault as usize]
            .set_exception_handler_with_error(general_protection_fault);
        idt.0[ExceptionIndex::PageFault as usize].set_exception_handler_with_error(page_fault);
        idt.0[ExceptionIndex::X87FloatingPointException as usize]
            .set_exception_handler(x87_floating_point_exception);
        idt.0[ExceptionIndex::AlignmentCheck as usize]
            .set_exception_handler_with_error(alignment_check);
        idt.0[ExceptionIndex::MachineCheck as usize].set_exception_handler(machine_check);
        idt.0[ExceptionIndex::SIMDFloatingPointException as usize]
            .set_exception_handler(simd_floating_point_exception);
        idt.0[ExceptionIndex::VirtualizationException as usize]
            .set_exception_handler(virtualization_exception);
        idt.0[ExceptionIndex::ControlProtectionException as usize]
            .set_exception_handler_with_error(control_protection_exception);
        idt.0[ExceptionIndex::HypervisorInjectionException as usize]
            .set_exception_handler(hypervisor_injection_exception);
        idt.0[ExceptionIndex::VMMCommunicationException as usize]
            .set_exception_handler_with_error(vmm_communication_exception);
        idt.0[ExceptionIndex::SecurityException as usize]
            .set_exception_handler_with_error(security_exception);

        // Hardware ISRs
        idt.0[32 + 0].set_interrupt_handler(timer_handler);
        idt.0[32 + 1].set_interrupt_handler(keyboard_handler);
        idt.0[32 + 12].set_interrupt_handler(mouse_handler);

        // Syscalls
        idt.0[64].set_interrupt_handler(print_interrupt);
        idt.0[64].set_dpl(PrivilegeLevel::Ring3);

        idt.0[65].set_interrupt_handler(put_screen_buffer);
        idt.0[65].set_dpl(PrivilegeLevel::Ring3);

        idt.0[66].set_interrupt_handler(get_screen_size);
        idt.0[66].set_dpl(PrivilegeLevel::Ring3);

        idt.0[67].set_interrupt_handler(load_file);
        idt.0[67].set_dpl(PrivilegeLevel::Ring3);

        idt.0[68].set_interrupt_handler(get_milliseconds_since_startup);
        idt.0[68].set_dpl(PrivilegeLevel::Ring3);

        idt.0[69].set_interrupt_handler(alloc_pages);
        idt.0[69].set_dpl(PrivilegeLevel::Ring3);

        idt.0[70].set_interrupt_handler(get_mouse_pos);
        idt.0[70].set_dpl(PrivilegeLevel::Ring3);

        idt.0[71].set_interrupt_handler(get_key);
        idt.0[71].set_dpl(PrivilegeLevel::Ring3);

        idt.0[72].set_interrupt_handler(exec);
        idt.0[72].set_dpl(PrivilegeLevel::Ring3);

        idt.0[73].set_interrupt_handler(get_shared_page);
        idt.0[73].set_dpl(PrivilegeLevel::Ring3);
    }
    let descriptor = IdtDescriptor::new(&IDT.lock());
    descriptor.load();

    Ok(())
}

#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,
    offset: u64,
}

impl IdtDescriptor {
    fn new(gdt: &Idt) -> IdtDescriptor {
        IdtDescriptor {
            size: size_of::<Idt>() as u16 - 1,
            offset: gdt as *const Idt as u64,
        }
    }

    fn load(&self) {
        unsafe {
            // Load IDT
            asm!("lidt [{register}]", register = in(reg) self);
        }
    }
}

#[repr(C)]
struct Idt([IdtEntry; 256]);

impl Idt {
    const fn new() -> Idt {
        Idt([IdtEntry::new(); 256])
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct IdtEntry(u64, u64);

impl IdtEntry {
    const fn new() -> IdtEntry {
        IdtEntry(0, 0)
    }

    fn set_exception_handler(&mut self, handler: extern "x86-interrupt" fn(InterruptStackFrame)) {
        self.set_offset(handler as u64);
        self.set_segment_selector(KERNEL_CODE_SEGMENT_SELECTOR as u16);
        self.set_interrupt_stack(1);
        self.set_gate_type(GateType::Trap);
        self.set_dpl(PrivilegeLevel::Ring0);
        self.set_present(true);
    }

    fn set_exception_handler_with_error(
        &mut self,
        handler: extern "x86-interrupt" fn(InterruptStackFrame, u64),
    ) {
        self.set_offset(handler as u64);
        self.set_segment_selector(KERNEL_CODE_SEGMENT_SELECTOR as u16);
        self.set_interrupt_stack(1);
        self.set_gate_type(GateType::Trap);
        self.set_dpl(PrivilegeLevel::Ring0);
        self.set_present(true);
    }

    fn set_interrupt_handler(&mut self, handler: extern "x86-interrupt" fn(InterruptStackFrame)) {
        self.set_offset(handler as u64);
        self.set_segment_selector(KERNEL_CODE_SEGMENT_SELECTOR as u16);
        self.set_interrupt_stack(1);
        self.set_gate_type(GateType::Interrupt);
        self.set_dpl(PrivilegeLevel::Ring0);
        self.set_present(true);
    }

    // Getters
    fn get_offset(&self) -> u64 {
        (self.0 & 0xffff) | ((self.0 >> (48 - 16)) & 0xffff_0000) | (self.1 << 32)
    }

    fn get_segment_selector(&self) -> u16 {
        (self.0 >> 16) as u16 & 0xffff
    }

    fn get_interrupt_stack(&self) -> u8 {
        (self.0 >> 32) as u8 & 0b111
    }

    fn get_gate_type(&self) -> GateType {
        GateType::new((self.0 >> 40) as u32 & 0xf)
    }

    fn get_dpl(&self) -> PrivilegeLevel {
        PrivilegeLevel::new((self.0 >> 45) as u32 & 0b11)
    }

    fn get_present(&self) -> bool {
        self.0 & (1 << 47) != 0
    }

    // Setters
    fn set_offset(&mut self, offset: u64) {
        self.0 &= !0xffff_0000_0000_ffff;
        self.1 &= !0xffff_ffff;

        self.0 |= (offset & 0xffff) | ((offset << (48 - 16)) & 0xffff_0000_0000_0000);
        self.1 |= offset >> 32;
    }

    fn set_segment_selector(&mut self, sel: u16) {
        self.0 &= !0xffff_0000;
        self.0 |= (sel as u64) << 16;
    }

    fn set_interrupt_stack(&mut self, interrupt_stack: u8) {
        self.0 &= !0x7_0000_0000;
        self.0 |= (interrupt_stack as u64 & 0b111) << 32;
    }

    fn set_gate_type(&mut self, t: GateType) {
        self.0 &= !0xf00_0000_0000;
        self.0 |= (t as u64) << 40;
    }

    fn set_dpl(&mut self, dpl: PrivilegeLevel) {
        self.0 &= !0x6000_0000_0000;
        self.0 |= (dpl as u64) << 45;
    }

    fn set_present(&mut self, present: bool) {
        self.0 &= !0x8000_0000_0000;
        self.0 |= (present as u64) << 47;
    }
}

enum GateType {
    Interrupt = 0xe,
    Trap = 0xf,
}

impl GateType {
    const fn new(value: u32) -> GateType {
        match value {
            0xe => Self::Interrupt,
            0xf => Self::Trap,
            _ => panic!("Invalid gate type"),
        }
    }
}

#[repr(usize)]
enum ExceptionIndex {
    DivisionError = 0,
    Debug,
    NonMaskableInterrupt,
    Breakpoint,
    Overflow,
    BoundRangeExceeded,
    InvalidOpcode,
    DeviceNotAvailable,
    DoubleFault,
    CoprocessorSegmentOverrun,
    InvalidTSS,
    SegmentNotPresent,
    StackSegmentFault,
    GeneralProtectionFault,
    PageFault,
    X87FloatingPointException = 16,
    AlignmentCheck,
    MachineCheck,
    SIMDFloatingPointException,
    VirtualizationException,
    ControlProtectionException,
    HypervisorInjectionException = 28,
    VMMCommunicationException,
    SecurityException,
}

#[repr(C)]
struct InterruptStackFrame {
    instruction_ptr: u64,
    code_segment: u64,
    r_flags: u64,
    stack_ptr: u64,
    stack_segment: u64,
}
