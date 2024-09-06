#![no_std]
#![no_main]

mod uefi;
mod utils;

use core::ffi::c_void;

#[inline]
#[panic_handler]
fn painc(_info: &core::panic::PanicInfo) -> ! {
    utils::halt();
}

#[no_mangle]
extern "efiapi" fn efi_main(_image_handle: *const c_void, system_table: *const uefi::SystemTable) {
    unsafe {
        ((*(*system_table).con_out).clear_screen)((*system_table).con_out);
    }

    utils::halt();
}
