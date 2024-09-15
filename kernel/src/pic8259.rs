#![allow(unused)]

use super::println;
use crate::utils::*;
use core::arch::asm;

const MASTER_PIC_COMMAND: u16 = 0x20;
const MASTER_PIC_DATA: u16 = 0x21;
const SLAVE_PIC_COMMAND: u16 = 0xa0;
const SLAVE_PIC_DATA: u16 = 0xa1;

pub fn init() -> Result<(), ()> {
    // TODO add checks to see if the PIC is present
    remap_pic(32, 32 + 8);
    enable_pic();
    Ok(())
}

fn remap_pic(master_offset: u8, slave_offset: u8) {
    // Save interrupt masks
    let master_mask: u8 = inb(MASTER_PIC_DATA);
    let slave_mask: u8 = inb(SLAVE_PIC_DATA);

    // Start initialization sequence in cascade mode
    outb(MASTER_PIC_COMMAND, 0x11);
    wait_io();
    outb(SLAVE_PIC_COMMAND, 0x11);
    wait_io();

    // Set new interrupt offsets
    outb(MASTER_PIC_DATA, master_offset);
    wait_io();
    outb(SLAVE_PIC_DATA, slave_offset);
    wait_io();

    // Specify relationship between master and slave
    outb(MASTER_PIC_DATA, 4);
    wait_io();
    outb(SLAVE_PIC_DATA, 2);
    wait_io();

    // Specify 8086 operation mode (instead of 8080)
    outb(MASTER_PIC_DATA, 1);
    wait_io();
    outb(SLAVE_PIC_DATA, 1);
    wait_io();

    // Restore interrupt masks
    outb(MASTER_PIC_DATA, master_mask);
    outb(SLAVE_PIC_DATA, slave_mask);
}

fn enable_pic() {
    // Set masks
    outb(MASTER_PIC_DATA, !0b110);
    outb(SLAVE_PIC_DATA, !0b10000);

    unsafe {
        asm!("sti");
    }
}

pub fn enable_irq(irq: u8) {
    if irq >= 8 {
        let m = inb(SLAVE_PIC_DATA);
        outb(SLAVE_PIC_DATA, m & !(1 << irq));
    } else {
        let m = inb(MASTER_PIC_DATA);
        outb(MASTER_PIC_DATA, m & !(1 << irq));
    }
}

fn disable_pic() {
    // Set masks
    outb(MASTER_PIC_DATA, 0xff);
    outb(SLAVE_PIC_DATA, 0xff);
}

#[inline]
pub fn end_of_interrupt(irq: u8) {
    if irq >= 8 {
        outb(SLAVE_PIC_COMMAND, 0x20);
    }

    outb(MASTER_PIC_COMMAND, 0x20);
}
