#![no_std]
#![allow(unused)]
#![feature(str_from_raw_parts)]
#![feature(const_mut_refs)]

pub mod fs;
pub mod graphics;
pub mod heap;

pub extern crate alloc;

use fs::*;
use graphics::*;

use alloc::vec::*;
use alloc::*;
use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    println!("USERSPACE: {}", panic_info);
    loop {}
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        STDOUT.write_fmt(args).unwrap();
    }
}

struct StdOut;

static mut STDOUT: StdOut = StdOut;

impl core::fmt::Write for StdOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            asm!(
                "int 0x40",
                in("rax") s.as_ptr(),
                in("rcx") s.len()
            );
        }
        Ok(())
    }
}

pub fn get_milliseconds_since_startup() -> u64 {
    let mut ms: u64;
    unsafe {
        asm!("int 0x44", out("rax") ms);
    }
    ms
}

pub fn alloc_pages(page_count: u64) -> u64 {
    let mut addr: u64;
    unsafe {
        asm!("int 0x45", in("rax") page_count, out("rcx") addr);
    }
    addr
}
