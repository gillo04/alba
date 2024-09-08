#![no_std]
#![no_main]

mod gdt;
mod memory;
mod stdout;
mod uefi;
mod utils;

use core::ffi::c_void;
use memory::MemoryManager;
use spin::mutex::Mutex;
use stdout::StdOut;
use uefi::SystemTable;

use crate::uefi::exit_boot_services;

static mut SYSTEM_TABLE: Mutex<*const SystemTable> = Mutex::new(0 as *const SystemTable);
static STDOUT: Mutex<StdOut> = Mutex::new(StdOut::new(0, 0, 0, 0, None));
static MEMORY_MANAGER: Mutex<MemoryManager> = Mutex::new(MemoryManager::new());

#[panic_handler]
fn painc(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    utils::halt();
}

#[no_mangle]
extern "efiapi" fn efi_main(image_handle: *const c_void, system_table: *const SystemTable) {
    // Get system table
    unsafe {
        *SYSTEM_TABLE.lock() = system_table;
    }

    // Initialize stdout
    stdout::init(system_table, None).expect("Failed to setup console");
    println!("Console setup\t\t\t\t[ \\gSUCCESS\\w ]");

    // Get memory map
    let memory_map_key = memory::init(system_table).expect("Failed to get memory map");
    println!("Got memory map\t\t\t\t[ \\gSUCCESS\\w ]");

    // Exit boot services
    exit_boot_services(system_table, image_handle, memory_map_key)
        .expect("Failed to exit boot services");
    println!("Exited boot services\t\t[ \\gSUCCESS\\w ]");

    // Initialize GDT
    gdt::init().expect("Failed to initialize GDT");
    println!("GDT setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    utils::halt();
}
