#![allow(unused)]

pub mod heap;
mod paging;

use super::{println, Mutex};
use crate::alloc::vec::*;
use crate::stdout::STDOUT;
use crate::uefi::*;
use crate::utils::clear_page;
use core::arch::asm;
use paging::*;

const KERNEL_BASE: u64 = 0x3333_0000_0000;
pub static MEMORY_MANAGER: Mutex<MemoryManager> = Mutex::new(MemoryManager::new());
pub static KERNEL_VALLOCATOR: Mutex<VirtualAllocator> =
    Mutex::new(VirtualAllocator::new(KERNEL_BASE));

// Initializes physical memory map. If successful returns the memory map key
pub fn init_physical(system_table: *const SystemTable) -> Result<usize, Status> {
    // Get memory map
    let result = MEMORY_MANAGER.lock().physical_map.init(system_table)?;

    Ok(MEMORY_MANAGER.lock().physical_map.key)
}

pub fn init_virtual(system_table: *const SystemTable) -> Result<(), Status> {
    // UEFI memory map
    let result = MEMORY_MANAGER
        .lock()
        .physical_map
        .uefi_identity_map(system_table)?;

    for _ in 0..1000 {
        MEMORY_MANAGER.lock().physical_map.alloc_frame();
    }

    // OS memory map
    let plm4 = MEMORY_MANAGER.lock().physical_map.alloc_frame();
    clear_page(plm4);
    let plm4 = unsafe { &mut *(plm4 as *mut PageTable) };
    // plm4.init();

    let descriptor_size = MEMORY_MANAGER.lock().physical_map.descriptor_size;
    let descriptor_count = MEMORY_MANAGER.lock().physical_map.mm_size / descriptor_size;
    let map = MEMORY_MANAGER.lock().physical_map.map;
    for i in 0..descriptor_count {
        let descriptor = unsafe {
            &*((&map as *const MemoryDescriptor as u64 + i as u64 * descriptor_size as u64)
                as *const MemoryDescriptor)
        };

        for i in 0..descriptor.number_of_pages {
            plm4.map(
                descriptor.physical_start + i * 0x1000,
                descriptor.physical_start + i * 0x1000,
                3,
            );
        }
    }

    // Map framebuffer
    {
        let s = STDOUT.lock();
        let fb_base = s.frame_buffer.base;
        let fb_page_count =
            s.frame_buffer.pixels_per_scanline * s.frame_buffer.height * 4 / 0x1000 + 1;
        for i in 0..fb_page_count {
            plm4.map(fb_base + i * 0x1000, fb_base + i * 0x1000, 3);
        }
    }

    MEMORY_MANAGER.lock().set_plm4(plm4);
    Ok(())
}

pub struct MemoryManager {
    pub physical_map: PhysicalMemoryMap,
    pub kernel_alloc_count: u64,
}

impl MemoryManager {
    const fn new() -> MemoryManager {
        MemoryManager {
            physical_map: PhysicalMemoryMap::new(),
            kernel_alloc_count: 0,
        }
    }

    pub fn get_plm4(&self) -> &'static mut PageTable {
        let out: u64;
        unsafe {
            asm!(
                "mov {}, cr3",
                out(reg) out
            );

            &mut *((out & !0xfff) as *mut PageTable)
        }
    }

    pub fn set_plm4(&self, plm4: &PageTable) {
        unsafe {
            asm!(
                "mov cr3, {}",
                in(reg) plm4 as *const PageTable as u64
            );
        }
    }
}

pub struct PhysicalMemoryMap {
    map: [MemoryDescriptor; 512],
    mm_size: usize,
    key: usize,
    descriptor_size: usize,
    descriptor_version: u32,

    alloc_count: u64,
}

impl PhysicalMemoryMap {
    const fn new() -> PhysicalMemoryMap {
        PhysicalMemoryMap {
            map: [MemoryDescriptor::new(); 512],
            mm_size: size_of::<[MemoryDescriptor; 512]>(),
            key: 0,
            descriptor_size: 0,
            descriptor_version: 0,
            alloc_count: 0,
        }
    }

    // Calls get_memory_map
    fn init(&mut self, system_table: *const SystemTable) -> Result<(), Status> {
        let status = unsafe {
            ((*(*system_table).boot_services).get_memory_map)(
                &mut self.mm_size as *mut usize,
                &mut self.map as *mut MemoryDescriptor,
                &mut self.key as *mut usize,
                &mut self.descriptor_size as *mut usize,
                &mut self.descriptor_version as *mut u32,
            )
        };

        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status),
        }
    }

    // Calls set_virtual_address_map
    fn uefi_identity_map(&mut self, system_table: *const SystemTable) -> Result<(), Status> {
        let descriptor_count = self.mm_size / self.descriptor_size;
        for i in 0..descriptor_count {
            let descriptor = unsafe {
                &mut *((&self.map as *const MemoryDescriptor as u64
                    + i as u64 * self.descriptor_size as u64)
                    as *mut MemoryDescriptor)
            };

            descriptor.virtual_start = descriptor.physical_start;
        }

        let status = unsafe {
            ((*(*system_table).runtime_services).set_virtual_address_map)(
                self.mm_size,
                self.descriptor_size,
                self.descriptor_version,
                &self.map as *const MemoryDescriptor,
            )
        };

        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status),
        }
    }

    pub fn alloc_frame(&mut self) -> u64 {
        let descriptor_count = self.mm_size / self.descriptor_size;
        let mut page_count = 0;
        for i in 0..descriptor_count {
            let descriptor = unsafe {
                &*((&self.map as *const MemoryDescriptor as u64
                    + i as u64 * self.descriptor_size as u64)
                    as *const MemoryDescriptor)
            };

            if descriptor.t == MemoryType::ConventionalMemory {
                if page_count + descriptor.number_of_pages > self.alloc_count {
                    let allocated_frame =
                        descriptor.physical_start + (self.alloc_count - page_count) * 0x1000;
                    self.alloc_count += 1;
                    return allocated_frame;
                } else {
                    page_count += descriptor.number_of_pages;
                }
            }
        }

        panic!("No more usable memory");
    }
}

#[derive(Debug)]
pub struct VirtualMapping {
    pub vaddr: u64,
    pub frames: Vec<u64>,
}

impl VirtualMapping {
    pub fn new(vaddr: u64, frames: Vec<u64>) -> VirtualMapping {
        VirtualMapping { vaddr, frames }
    }
}

pub struct VirtualAllocator {
    vaddr: u64,
    alloc_count: u64,
}

impl VirtualAllocator {
    const fn new(vaddr: u64) -> VirtualAllocator {
        VirtualAllocator {
            vaddr,
            alloc_count: 0,
        }
    }

    pub fn alloc_pages(&mut self, page_count: u64) -> VirtualMapping {
        let mut out = VirtualMapping::new(
            self.vaddr + self.alloc_count * 0x1000,
            Vec::with_capacity(page_count as usize),
        );
        for i in 0..page_count {
            out.frames
                .push(MEMORY_MANAGER.lock().physical_map.alloc_frame());
            clear_page(out.frames[i as usize]);
        }
        let plm4 = MEMORY_MANAGER.lock().get_plm4();
        plm4.map_mapping(&out);
        self.alloc_count += page_count;
        out
    }
}
