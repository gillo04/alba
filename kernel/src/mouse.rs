#![allow(unused)]

use super::{println, Mutex};
use crate::utils::*;
use core::arch::asm;

// Adapted from the zeitgeist of mouse drivers

const MOUSE_PORT: u16 = 0x60;
const MOUSE_STATUS: u16 = 0x64;
const MOUSE_ABIT: u8 = 0x02;
const MOUSE_BBIT: u8 = 0x01;
const MOUSE_WRITE: u8 = 0xD4;
const MOUSE_F_BIT: u16 = 0x20;
const MOUSE_V_BIT: u16 = 0x08;

pub static MOUSE_POS: Mutex<(u64, u64, bool, bool)> = Mutex::new((0, 0, false, false));

pub fn init() -> Result<(), &'static str> {
    let mut status: u8 = 0;

    unsafe {
        asm!("cli");
    }
    mouse_wait(true)?;
    outb(MOUSE_STATUS, 0xA8);
    mouse_wait(true)?;
    outb(MOUSE_STATUS, 0x20);
    mouse_wait(false)?;
    status = inb(0x60) | 2;
    mouse_wait(true)?;
    outb(MOUSE_STATUS, 0x60);
    mouse_wait(true)?;
    outb(MOUSE_PORT, status);
    mouse_write(0xF6)?;
    mouse_read()?;
    mouse_write(0xF4)?;
    mouse_read()?;
    unsafe {
        asm!("sti");
    }

    Ok(())
}

fn mouse_wait(a_type: bool) -> Result<(), &'static str> {
    let mut timeout = 100000;
    if !a_type {
        while timeout > 0 {
            if inb(MOUSE_STATUS) & MOUSE_BBIT == 1 {
                return Ok(());
            }
            timeout -= 1;
        }
    } else {
        while timeout > 0 {
            if inb(MOUSE_STATUS) & MOUSE_ABIT != 0 {
                return Ok(());
            }
            timeout -= 1;
        }
    }
    // Err("Mouse timeout")
    Ok(())
}

fn mouse_write(write: u8) -> Result<(), &'static str> {
    mouse_wait(true)?;
    outb(MOUSE_STATUS, MOUSE_WRITE);
    mouse_wait(true)?;
    outb(MOUSE_PORT, write);
    Ok(())
}

fn mouse_read() -> Result<u8, &'static str> {
    mouse_wait(false)?;
    Ok(inb(MOUSE_PORT))
}
