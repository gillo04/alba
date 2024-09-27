#![no_std]
#![allow(unused)]
#![feature(str_from_raw_parts)]
#![feature(const_mut_refs)]

pub mod desktop;
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

pub fn get_mouse_position() -> (u64, u64) {
    let mut out: (u64, u64) = (0, 0);
    unsafe {
        asm!("int 0x46", out("rax") out.0, out("rcx") out.1);
    }
    out
}

pub fn get_key() -> Option<(u8, u8)> {
    let mut present: u64 = 0;
    let mut c: u64 = 0;
    let mut sc: u64 = 0;
    unsafe {
        asm!("int 0x47", out("rax") present, out("rcx") c, out("rdx") sc);
    }

    if present == 0 {
        return None;
    } else {
        return Some((c as u8, sc as u8));
    }
}

pub fn exec(path: &str) -> Result<(), ()> {
    let mut res: u64 = 0;
    unsafe {
        asm!(
            "int 0x48",
            in("rax") path.as_ptr(),
            in("rcx") path.len(),
            out("rdx") res,
        );
    }
    if res == 0 {
        Err(())
    } else {
        Ok(())
    }
}

pub fn get_shared_page() -> u64 {
    let mut ptr: u64 = 0;
    unsafe {
        asm!(
            "int 0x49",
            out("rax") ptr,
        );
    }
    ptr
}
