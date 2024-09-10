#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    halt();
}

#[export_name = "_start"]
#[no_mangle]
fn main() {
    halt();
}

#[inline]
fn halt() -> ! {
    loop {}
}
