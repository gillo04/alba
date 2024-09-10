#![allow(unused)]

use super::{print, println, Mutex};
use crate::ata::*;
use crate::drive::Drive;
use crate::fs::*;
use crate::memory::{VirtualMapping, KERNEL_VALLOCATOR, MEMORY_MANAGER};
use crate::utils::clear_page;
use alloc::string::*;
use alloc::vec::*;

pub static FAT32: Mutex<Option<Fat32Fs<AtaDrive48>>> = Mutex::new(None);

pub fn init() -> Result<(), ()> {
    *FAT32.lock() = Some(Fat32Fs::new(*crate::ata::ATA_DRIVE_48.lock()));
    Ok(())
}

pub struct Fat32Fs<D: Drive> {
    drive: D,
    boot_sector: Fat32BootSector,
    fat_buffer: u64,
}

impl<D: Drive> Fat32Fs<D> {
    pub fn new(drive: D) -> Fat32Fs<D> {
        let boot_sector = Fat32BootSector::new(&drive);

        // Load fat into memory
        let first_fat_sector = boot_sector.bpd.reserved_sector_count;
        let fat_size = boot_sector.ebpb.table_size_32;
        let fat_buffer = KERNEL_VALLOCATOR
            .lock()
            .alloc_pages(fat_size as u64 * boot_sector.bpd.bytes_per_sector as u64 / 0x1000 + 1)
            .vaddr;

        drive.read_sectors(
            first_fat_sector as u64,
            fat_size as u64,
            fat_buffer as *mut u8,
        );

        Fat32Fs {
            drive,
            boot_sector,
            fat_buffer,
        }
    }

    fn cluster_to_sector(&self, cluster: u32) -> u64 {
        let start_of_data = self.boot_sector.bpd.reserved_sector_count as u64
            + self.boot_sector.bpd.table_count as u64 * self.boot_sector.ebpb.table_size_32 as u64;

        let sector = start_of_data as i64
            + (cluster as i64 - 2) * self.boot_sector.bpd.sectors_per_cluster as i64;
        sector as u64
    }

    pub fn read_directory(&self, cluseter: u32) -> &[StandardDirectory] {
        let buffer = MEMORY_MANAGER.lock().physical_map.alloc_frame() as *const StandardDirectory;

        self.drive.read_sectors(
            self.cluster_to_sector(cluseter) as u64,
            self.boot_sector.bpd.sectors_per_cluster as u64,
            buffer as *mut u8,
        );

        let mut max_entries = self.boot_sector.bpd.sectors_per_cluster as u16
            * self.boot_sector.bpd.bytes_per_sector
            / size_of::<StandardDirectory>() as u16;

        for i in 0..max_entries {
            let base = unsafe { buffer.offset(i as isize) };
            let byte_array = base as *const u8;

            if unsafe { *byte_array } == 0 {
                max_entries = i;
                break;
            }
        }

        unsafe { core::slice::from_raw_parts(buffer, max_entries as usize) }
    }

    fn path_to_cluster(&self, path: &str) -> Result<u32, &str> {
        self.recursive_path_to_cluster(path.split("/").peekable(), 2)
    }

    // TODO rewrite it as non recursive
    fn recursive_path_to_cluster(
        &self,
        mut path: core::iter::Peekable<core::str::Split<&str>>,
        cluster: u32,
    ) -> Result<u32, &str> {
        let this = path.next();
        if this == None {
            return Err("File not found");
        }
        let this = this.unwrap();

        let next = path.peek();
        let directory = self.read_directory(cluster);
        for entry in directory.iter() {
            let entry_name = core::str::from_utf8(&entry.filename).unwrap().trim();
            let next_cluster =
                ((entry.first_cluster_high as u32) << 16) | entry.first_cluster_low as u32;
            if entry_name == this {
                if next.is_some() {
                    return self.recursive_path_to_cluster(path, next_cluster);
                } else {
                    return Ok(next_cluster);
                }
            }
        }

        return Err("File not found");
    }

    fn read_cluster_chain(&self, cluster: u32) -> VirtualMapping {
        // Create file buffer
        let mut current_cluster = cluster;
        let mut cluster_count = 0;
        while current_cluster < 0xFFFFFF8 {
            cluster_count += 1;
            current_cluster =
                unsafe { *(self.fat_buffer as *const u32).offset(current_cluster as isize) };
        }

        let page_count = cluster_count
            * self.boot_sector.bpd.sectors_per_cluster as u64
            * self.boot_sector.bpd.bytes_per_sector as u64
            / 0x1000
            + 1;
        let mut file_mapping = KERNEL_VALLOCATOR.lock().alloc_pages(page_count);

        // Follow cluster chain
        let mut current_cluster = cluster;
        let mut file_buffer_offset = 0;
        while current_cluster < 0xFFFFFF8 {
            self.drive.read_sectors(
                self.cluster_to_sector(current_cluster),
                self.boot_sector.bpd.sectors_per_cluster as u64,
                (file_mapping.vaddr + file_buffer_offset) as *mut u8,
            );

            file_buffer_offset += self.boot_sector.bpd.sectors_per_cluster as u64
                * self.boot_sector.bpd.bytes_per_sector as u64;
            current_cluster =
                unsafe { *(self.fat_buffer as *const u32).offset(current_cluster as isize) };
        }

        file_mapping
    }

    // Depth first search
    pub fn dfs(&self, clusrer: u32, depth: u32) {
        let directory = self.read_directory(clusrer);
        for file in directory {
            for _ in 0..depth {
                print!(" ");
            }

            let name = core::str::from_utf8(&file.filename).unwrap().trim();
            if name != "."
                && name != ".."
                && file.attributes as u32 & StandardDirectoryAttributes::Hidden as u32 == 0
            {
                println!("{}", core::str::from_utf8(&file.filename).unwrap());
                if file.attributes as u32 & StandardDirectoryAttributes::Directory as u32 != 0 {
                    self.dfs(
                        file.first_cluster_low as u32 | ((file.first_cluster_high as u32) << 16),
                        depth + 1,
                    );
                }
            }
        }
    }
}

impl<D: Drive> Fs for Fat32Fs<D> {
    fn read_file(&self, path: &str) -> Result<VirtualMapping, &str> {
        let cluster = self.path_to_cluster(path)?;
        Ok(self.read_cluster_chain(cluster))
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Fat32BootSector {
    pub bpd: FatBpb,
    pub ebpb: Fat32Ebpd,
}

impl Fat32BootSector {
    pub fn new<D: Drive>(drive: &D) -> Fat32BootSector {
        let buffer = MEMORY_MANAGER.lock().physical_map.alloc_frame();
        clear_page(buffer);
        drive.read_sectors(0, 1, buffer as *mut u8);
        let boot_sector = unsafe { *(buffer as *const Fat32BootSector) };
        boot_sector
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct FatBpb {
    pub bootjmp: [u8; 3],
    pub oem_name: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub table_count: u8,
    pub root_entry_count: u16,
    pub total_sectors_16: u16,
    pub media_type: u8,
    pub table_size_16: u16,
    pub sectors_per_track: u16,
    pub head_side_count: u16,
    pub hidden_sector_count: u32,
    pub total_sectors_32: u32,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Fat32Ebpd {
    pub table_size_32: u32,
    pub extended_flags: u16,
    pub fat_version: u16,
    pub root_cluster: u32,
    pub fat_info: u16,
    pub backup_bs_sector: u16,
    pub _reserved0: [u8; 12],
    pub drive_number: u8,
    pub _reserved1: u8,
    pub boot_signature: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fat_type_label: [u8; 8],
}

#[repr(C, packed)]
pub struct StandardDirectory {
    pub filename: [u8; 11],
    pub attributes: StandardDirectoryAttributes,
    pub reserved_by_windows: u8,
    pub creation_time_hundredths: u8,
    pub creation_time: u16,
    pub creation_date: u16,
    pub last_accessed_date: u16,
    pub first_cluster_high: u16,
    pub last_modification_time: u16,
    pub last_modification_date: u16,
    pub first_cluster_low: u16,
    pub file_size_bytes: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum StandardDirectoryAttributes {
    ReadOnly = 0x01,
    Hidden = 0x02,
    System = 0x04,
    VolumeId = 0x08,
    Directory = 0x10,
    Archive = 0x20,
    Lfn = 0x3f,
}
