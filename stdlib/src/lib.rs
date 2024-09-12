#![no_std]

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
