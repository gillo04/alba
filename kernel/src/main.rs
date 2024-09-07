#![no_std]
#![no_main]

mod stdout;
mod uefi;
mod utils;

use core::ffi::c_void;
use spin::mutex::Mutex;
use stdout::*;
use uefi::SystemTable;

static mut SYSTEM_TABLE: Mutex<*const SystemTable> = Mutex::new(0 as *const SystemTable);
static STDOUT: Mutex<StdOut> = Mutex::new(StdOut::new(0, 0, 0, 0, None));

#[panic_handler]
fn painc(_info: &core::panic::PanicInfo) -> ! {
    utils::halt();
}

#[no_mangle]
extern "efiapi" fn efi_main(_image_handle: *const c_void, system_table: *const SystemTable) {
    // Get system table
    unsafe {
        *SYSTEM_TABLE.lock() = system_table;
    }

    // Initialize stdout
    stdout::init(None);

    println!("Console setup\t\t\t\t[ \\gSUCCESS\\w ]");

    utils::halt();
}
