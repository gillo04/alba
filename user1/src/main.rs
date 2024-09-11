#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

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

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    loop {}
}

#[export_name = "_start"]
#[no_mangle]
fn main() {
    println!("\\rNumbers from 0 to 10:\\w");
    for i in 0..10 {
        println!("\\r\t{}\\w", i);
    }
    loop {}
}
