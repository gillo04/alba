#![allow(unused)]

mod paging;

use super::{println, MEMORY_MANAGER};
use crate::uefi::*;

const KERNEL_BASE: usize = 0x3333_0000_0000;
const PAGE_TABLES_BASE: usize = 0x2222_0000_0000;
const KERNEL_HEAP: usize = 0x1111_0000_0000;

// Initializes physical memory map. If successful returns the memory map key
pub fn init_physical(system_table: *const SystemTable) -> Result<usize, Status> {
    // Get memory map
    let result = MEMORY_MANAGER.lock().physical_map.init(system_table)?;

    Ok(MEMORY_MANAGER.lock().physical_map.key)
}

pub fn init_virtual(system_table: *const SystemTable) -> Result<(), Status> {
    let result = MEMORY_MANAGER
        .lock()
        .physical_map
        .uefi_identity_map(system_table)?;
    Ok(())
}

pub struct MemoryManager {
    pub physical_map: PhysicalMemoryMap,
}

impl MemoryManager {
    pub const fn new() -> MemoryManager {
        MemoryManager {
            physical_map: PhysicalMemoryMap::new(),
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
