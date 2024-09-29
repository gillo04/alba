#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(str_from_raw_parts)]

extern crate alloc;
use alloc::string::*;

mod ata;
mod drive;
mod elf;
mod fat32;
mod fs;
mod gdt;
mod idt;
mod memory;
mod mouse;
mod pic8259;
mod pit;
mod process;
mod stdin;
mod stdout;
mod uefi;
mod utils;

use crate::memory::*;
use core::arch::asm;
use core::ffi::c_void;
use elf::ElfExecutable;
use fat32::*;
use fs::*;
use process::*;
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

    // Initialize heap
    memory::heap::init().expect("Failed to initialize heap");
    println!("Heap setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    // Initialize GDT
    gdt::init().expect("Failed to initialize GDT");
    println!("GDT setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    // Initialize IDT
    idt::init().expect("Failed to initialize IDT");
    println!("IDT setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    // Initialize PIT
    pit::init().expect("Failed to initialize PIT");
    println!("PIT setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    // Initialize PIC
    pic8259::init().expect("Failed to initialize PIC");
    println!("PIC setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    // Identify ATA drive
    ata::init().expect("Failed to identify primary master drive");
    println!("ATA drive identified\t\t[ \\gSUCCESS\\w ]");

    // Initialize FAT32 file system
    fat32::init().expect("Failed to initialize FAT32 file system");
    println!("FAT32 setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    // Initialize mouse
    mouse::init().expect("Failed to initialize mouse");
    println!("Mouse setup\t\t\t\t\t[ \\gSUCCESS\\w ]");

    // Start userspace program
    let desktop = FAT32
        .lock()
        .as_ref()
        .unwrap()
        .read_file("USER/DESKTOP")
        .unwrap();
    let desktop = ElfExecutable::new(desktop);
    let desktop = Process::new(desktop.load_all(), desktop.get_entry());
    PROCESS_LIST.lock().processes.push(desktop);

    println!("Elf files loaded");

    let shared_page = KERNEL_VALLOCATOR.lock().alloc_pages(1000);
    MEMORY_MANAGER
        .lock()
        .get_plm4()
        .map_mapping_user(&shared_page);
    *SHARED_PAGE.lock() = shared_page.vaddr;
    PROCESS_LIST.lock().jump_to_multitasking = true;

    pic8259::enable_irq(0);

    utils::halt();
}
