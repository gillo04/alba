use super::MEMORY_MANAGER;
use crate::uefi::*;

// If successful returns the memory map key
pub fn init(system_table: *const SystemTable) -> Result<usize, Status> {
    // Get memory map
    let result = MEMORY_MANAGER.lock().uefi_map.init(system_table);
    match result {
        Ok(()) => Ok(MEMORY_MANAGER.lock().uefi_map.key),
        Err(status) => Err(status),
    }
}

pub struct MemoryManager {
    pub uefi_map: UefiMemoryMap,
}

impl MemoryManager {
    pub const fn new() -> MemoryManager {
        MemoryManager {
            uefi_map: UefiMemoryMap::new(),
        }
    }
}

pub struct UefiMemoryMap {
    map: [MemoryDescriptor; 512],
    mm_size: usize,
    key: usize,
    descriptor_size: usize,
    descriptor_ver: u32,
}

impl UefiMemoryMap {
    pub const fn new() -> UefiMemoryMap {
        UefiMemoryMap {
            map: [MemoryDescriptor::new(); 512],
            mm_size: size_of::<[MemoryDescriptor; 512]>(),
            key: 0,
            descriptor_size: 0,
            descriptor_ver: 0,
        }
    }

    // Calls get_memory_map
    pub fn init(&mut self, system_table: *const SystemTable) -> Result<(), Status> {
        let status = unsafe {
            ((*(*system_table).boot_services).get_memory_map)(
                &mut self.mm_size as *mut usize,
                &mut self.map as *mut MemoryDescriptor,
                &mut self.key as *mut usize,
                &mut self.descriptor_size as *mut usize,
                &mut self.descriptor_ver as *mut u32,
            )
        };

        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status),
        }
    }
}
