#![allow(unused)]

pub trait Drive {
    fn read_sectors(&self, lba: u64, sector_count: u64, buffer: *mut u8);
}
