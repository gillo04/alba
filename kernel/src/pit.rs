#![allow(unused)]

use super::{println, Mutex};
use crate::utils::*;

pub static MILLISECONDS_SINCE_STARTUP: Mutex<u64> = Mutex::new(0);

const CHANNEL0_DATA: u16 = 0x40;
const CHANNEL1_DATA: u16 = 0x41;
const CHANNEL2_DATA: u16 = 0x42;
const COMMAND: u16 = 0x43;

const PIT_FREQUENCY: u32 = 1193182;

pub fn init() -> Result<(), ()> {
    let divisor = 2u16.pow(11);
    outb(COMMAND, 0x36);
    outb(CHANNEL0_DATA, divisor as u8);
    outb(CHANNEL0_DATA, (divisor >> 8) as u8);
    Ok(())
}
