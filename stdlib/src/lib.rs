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
use graphics::gui::*;
use graphics::text::*;
use graphics::*;

use alloc::string::*;
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
                "int 0x80",
                in("rax") 0x10,
                in("rcx") s.as_ptr(),
                in("rdx") s.len()
            );
        }
        Ok(())
    }
}

pub fn get_milliseconds_since_startup() -> u64 {
    let mut ms: u64;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x50,
            out("rcx") ms
        );
    }
    ms
}

pub fn alloc_pages(page_count: u64) -> u64 {
    let mut addr: u64;
    unsafe {
        asm!("int 0x80",
            in("rax") 0x40,
            in("rcx") page_count,
            out("rdx") addr
        );
    }
    addr
}

pub fn get_key() -> Option<(u8, u8)> {
    let mut present: u64 = 0;
    let mut c: u64 = 0;
    let mut sc: u64 = 0;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x20,
            out("rcx") present,
            out("rdx") c,
            out("r8") sc
        );
    }

    if present == 0 {
        return None;
    } else {
        return Some((c as u8, sc as u8));
    }
}

pub fn get_mouse() -> (u64, u64, bool, bool) {
    let mut out: (u64, u64, bool, bool) = (0, 0, false, false);
    let mut l: u64;
    let mut r: u64;
    unsafe {
        asm!("int 0x80",
            in("rax") 0x21,
            out("rcx") out.0,
            out("rdx") out.1,
            out("r8") l,
            out("r9") r
        );
    }
    out.2 = l != 0;
    out.3 = r != 0;
    out
}

// Returns the pid of the spawned process
pub fn exec(path: &str) -> Result<u32, ()> {
    let mut res: u64 = 0;
    let mut pid: u64 = 0;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x60,
            in("rcx") path.as_ptr(),
            in("rdx") path.len(),
            out("r8") res,
            out("r9") pid,
        );
    }

    if res == 0 {
        Err(())
    } else {
        Ok(pid as u32)
    }
}

pub fn get_shared_page() -> u64 {
    let mut ptr: u64 = 0;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x41,
            out("rcx") ptr,
        );
    }
    ptr
}

pub fn exit() {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x61,
        );
    }
}

pub fn kill(pid: u32) {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") 0x62,
            in("rcx") pid as u64,
        );
    }
}
