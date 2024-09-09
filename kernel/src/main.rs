#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod gdt;
mod idt;
mod memory;
mod pic8259;
mod stdout;
mod uefi;
mod utils;

use core::arch::asm;
use core::ffi::c_void;
use spin::mutex::Mutex;
use uefi::SystemTable;

use crate::uefi::exit_boot_services;

static mut SYSTEM_TABLE: Mutex<*const SystemTable> = Mutex::new(0 as *const SystemTable);

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
    stdout::init(system_table, None).expect("Failed to initialize console");
    println!("Console setup\t\t\t\t[ \\gSUCCESS\\w ]");

    // Get memory map
    let memory_map_key = memory::init_physical(system_table).expect("Failed to get memory map");
    println!("Got memory map\t\t\t\t[ \\gSUCCESS\\w ]");

    // Exit boot services
    exit_boot_services(system_table, image_handle, memory_map_key)
        .expect("Failed to exit boot services");
    println!("Exited boot services\t\t[ \\gSUCCESS\\w ]");

    // Initialize virtual memory
    memory::init_virtual(system_table).expect("Failed to initialize virtual memory");
    println!("Virtual memory setup\t\t[ \\gSUCCESS\\w ]");

    // Initialize GDT
    gdt::init().expect("Failed to initialize GDT");
    println!("GDT setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    // Initialize IDT
    idt::init().expect("Failed to initialize IDT");
    println!("IDT setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    // Initialize PIC
    pic8259::init().expect("Failed to initialize PIC");
    println!("PIC setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    utils::halt();
}
