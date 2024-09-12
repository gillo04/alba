#![no_std]
#![allow(unused)]

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
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

pub struct Circle {
    pub x: i64,
    pub y: i64,
    pub r: u64,
    pub color: u32,
}

impl Draw for Circle {
    fn draw(&self, sb: &ScreenBuffer) {
        // Bounds checking
        let rect_x = i64::clamp(self.x - self.r as i64, 0, sb.w as i64);
        let rect_y = i64::clamp(self.y - self.r as i64, 0, sb.h as i64);
        let rect_right = i64::clamp(self.x + self.r as i64, 0, sb.w as i64);
        let rect_bottom = i64::clamp(self.y + self.r as i64, 0, sb.h as i64);

        let r2 = self.r.pow(2);
        for i in rect_x..rect_right {
            for j in rect_y..rect_bottom {
                let distance = (j - self.x).pow(2) + (i - self.y).pow(2);
                if distance < r2 as i64 {
                    unsafe {
                        *(sb.base as *mut u32).offset((i * sb.w as i64 + j) as isize) = self.color;
                    }
                }
            }
        }
    }
}

pub struct Rectangle {
    pub x: i64,
    pub y: i64,
    pub w: u64,
    pub h: u64,
    pub color: u32,
}

impl Draw for Rectangle {
    fn draw(&self, sb: &ScreenBuffer) {
        // Bounds checking
        if self.x >= sb.w as i64 || self.y >= sb.w as i64 {
            return;
        }
        let x = i64::max(self.x, 0);
        let y = i64::max(self.y, 0);
        let mut right = i64::min(self.x + self.w as i64, sb.w as i64);
        let mut bottom = i64::min(self.y + self.h as i64, sb.h as i64);

        for i in y..bottom {
            for j in x..right {
                unsafe {
                    *(sb.base as *mut u32).offset((i * sb.w as i64 + j) as isize) = self.color;
                }
            }
        }
    }
}

pub trait Draw {
    fn draw(&self, sb: &ScreenBuffer);
}

pub struct ScreenBuffer {
    pub x: u64,
    pub y: u64,
    pub w: u64,
    pub h: u64,
    pub base: u64,
}

impl ScreenBuffer {
    pub fn new(x: u64, y: u64, w: u64, h: u64, base: u64) -> ScreenBuffer {
        ScreenBuffer { x, y, w, h, base }
    }

    pub fn put(&self) {
        unsafe {
            asm!(
                "int 0x41",
                in("rax") self.base,
                in("rcx") self.x,
                in("rdx") self.y,
                in("r8") self.w,
                in("r9") self.h,
            );
        }
    }

    pub fn clear(&self, color: u32) {
        for i in 0..self.h {
            for j in 0..self.w {
                unsafe {
                    *(self.base as *mut u32).offset((i * self.w + j) as isize) = color;
                }
            }
        }
    }

    pub fn draw(&self, obj: &impl Draw) {
        obj.draw(self);
    }
}
