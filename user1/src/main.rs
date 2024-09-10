#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(_panic_info: &PanicInfo) -> ! {
    loop {}
}

#[export_name = "_start"]
#[no_mangle]
fn main() {
    unsafe {
        asm!("int3");
    }
    loop {}
}
