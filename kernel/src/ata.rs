#![allow(unused)]

use crate::drive::*;
use crate::utils::*;

const PRIMARY_IO_BASE: u16 = 0x1F0;
const PRIMARY_CONTROL_BASE: u16 = 0x3F6;

const SECONDARY_IO_BASE: u16 = 0x170;
const SECONDARY_CONTROL_BASE: u16 = 0x376;

#[derive(Clone, Copy)]
pub struct AtaDrive28<'a> {
    ata_bus: &'a AtaBus,
    drive_selector: DriveSelector,
}

impl Drive for AtaDrive28<'_> {
    fn read_sectors(&self, lba: u64, sector_count: u64, buffer: *mut u8) {
        let buffer = buffer as *mut u16;
        self.ata_bus.set_sector_count(sector_count as u8);
        self.ata_bus.set_lba_low(lba as u8);
        self.ata_bus.set_lba_mid((lba >> (8 * 1)) as u8);
        self.ata_bus.set_lba_high((lba >> (8 * 2)) as u8);
        match self.drive_selector {
            DriveSelector::Master => {
                self.ata_bus.set_drive(0xe0);
            }
            DriveSelector::Slave => {
                self.ata_bus.set_drive(0xf0);
            }
        }
        self.ata_bus.set_command(0x20);

        self.ata_bus.delay_400ns();
        if self.ata_bus.get_status() & 1 == 1 {
            panic!(
                "Error while reading drive {:?}: 0b{:b}",
                self.drive_selector,
                self.ata_bus.get_error()
            );
        }

        for sector in 0..sector_count {
            // Wait for the drive to be ready to transfer data
            self.ata_bus.wait_drq_set();
            self.ata_bus.wait_bsy_clear();

            for i in 0..256 {
                unsafe {
                    *buffer.offset(i as isize + sector as isize * 256) = self.ata_bus.get_data();
                }
            }
            self.ata_bus.delay_400ns();
        }
    }
}

#[derive(Clone, Copy)]
pub struct AtaDrive48<'a> {
    ata_bus: &'a AtaBus,
    drive_selector: DriveSelector,
}

impl Drive for AtaDrive48<'_> {
    fn read_sectors(&self, lba: u64, sector_count: u64, buffer: *mut u8) {
        let buffer = buffer as *mut u16;
        self.ata_bus.set_sector_count((sector_count >> 8) as u8);
        self.ata_bus.set_lba_low((lba >> (8 * 3)) as u8);
        self.ata_bus.set_lba_mid((lba >> (8 * 4)) as u8);
        self.ata_bus.set_lba_high((lba >> (8 * 5)) as u8);
        self.ata_bus.set_sector_count(sector_count as u8);
        self.ata_bus.set_lba_low(lba as u8);
        self.ata_bus.set_lba_mid((lba >> (8 * 1)) as u8);
        self.ata_bus.set_lba_high((lba >> (8 * 2)) as u8);
        match self.drive_selector {
            DriveSelector::Master => {
                self.ata_bus.set_drive(0x40);
            }
            DriveSelector::Slave => {
                self.ata_bus.set_drive(0x50);
            }
        }
        self.ata_bus.set_command(0x24);

        self.ata_bus.delay_400ns();
        if self.ata_bus.get_status() & 1 == 1 {
            panic!(
                "Error while reading drive {:?}: 0b{:b}",
                self.drive_selector,
                self.ata_bus.get_error()
            );
        }

        for sector in 0..sector_count {
            // Wait for the drive to be ready to transfer data
            self.ata_bus.wait_drq_set();
            self.ata_bus.wait_bsy_clear();

            for i in 0..256 {
                unsafe {
                    *buffer.offset(i as isize + sector as isize * 256) = self.ata_bus.get_data();
                }
            }
            self.ata_bus.delay_400ns();
        }
    }
}

#[derive(Clone, Copy)]
pub struct AtaBus {
    io_base: u16,
    control_base: u16,
}

#[allow(unused)]
impl AtaBus {
    pub fn primary() -> AtaBus {
        AtaBus {
            io_base: PRIMARY_IO_BASE,
            control_base: PRIMARY_CONTROL_BASE,
        }
    }

    pub fn secondary() -> AtaBus {
        AtaBus {
            io_base: SECONDARY_IO_BASE,
            control_base: SECONDARY_CONTROL_BASE,
        }
    }

    pub fn get_data(&self) -> u16 {
        inw(self.io_base)
    }

    pub fn set_data(&self, data: u16) {
        outw(self.io_base, data);
    }

    pub fn get_error(&self) -> u8 {
        inb(self.io_base + 1)
    }

    pub fn set_features(&self, features: u8) {
        outb(self.io_base + 1, features);
    }

    pub fn get_sector_count(&self) -> u8 {
        inb(self.io_base + 2)
    }

    pub fn set_sector_count(&self, sec_count: u8) {
        outb(self.io_base + 2, sec_count);
    }

    pub fn get_lba_low(&self) -> u8 {
        inb(self.io_base + 3)
    }

    pub fn set_lba_low(&self, lba: u8) {
        outb(self.io_base + 3, lba);
    }

    pub fn get_lba_mid(&self) -> u8 {
        inb(self.io_base + 4)
    }

    pub fn set_lba_mid(&self, lba: u8) {
        outb(self.io_base + 4, lba);
    }

    pub fn get_lba_high(&self) -> u8 {
        inb(self.io_base + 5)
    }

    pub fn set_lba_high(&self, lba: u8) {
        outb(self.io_base + 5, lba);
    }

    pub fn get_drive(&self) -> u8 {
        inb(self.io_base + 6)
    }

    pub fn set_drive(&self, drive: u8) {
        outb(self.io_base + 6, drive);
    }

    pub fn get_status(&self) -> u8 {
        inb(self.io_base + 7)
    }

    pub fn set_command(&self, command: u8) {
        outb(self.io_base + 7, command);
    }

    pub fn get_alternate_status(&self) -> u8 {
        inb(self.control_base)
    }

    pub fn set_device_control(&self, dev_ctrl: u8) {
        outb(self.control_base, dev_ctrl);
    }

    pub fn get_device_address(&self) -> u8 {
        inb(self.control_base + 1)
    }

    pub fn delay_400ns(&self) {
        for _ in 0..5 {
            self.get_status();
        }
    }

    pub fn wait_bsy_clear(&self) {
        while &self.get_status() & (1 << 7) != 0 {}
    }

    pub fn wait_drq_set(&self) {
        while &self.get_status() & (1 << 3) == 0 {}
    }

    pub fn flush_cache(&self) {
        self.set_command(0xe7);
        self.wait_bsy_clear();
    }

    pub fn identify(&self, drive: DriveSelector) -> Result<AtaDrive48, &str> {
        if drive == DriveSelector::Master {
            self.set_drive(0xa0);
        } else {
            self.set_drive(0xa0);
        }

        self.set_sector_count(0);
        self.set_lba_low(0);
        self.set_lba_mid(0);
        self.set_lba_high(0);

        self.set_command(0xec);
        self.delay_400ns();
        let exists = self.get_status() != 0;

        if exists {
            self.wait_bsy_clear();

            if self.get_lba_mid() != 0 && self.get_lba_high() != 0 {
                return Err("Drive does not support ATA");
            }

            // Whait for DRQ or for ERR to set
            while self.get_status() & 0b1001 == 0 {}
            if self.get_status() & 1 == 0 {
                let mut buffer = [0u16; 256];
                for i in 0..256 {
                    buffer[i] = self.get_data();
                }

                if buffer[83] & (1 << 10) != 0 {
                    return Ok(AtaDrive48 {
                        ata_bus: self,
                        drive_selector: drive,
                    });
                    /*return Ok(AtaDrive28 {
                        ata_bus: self,
                        drive_selector: drive,
                    });*/
                } else {
                    return Err("LBA28 unsupported");
                    /*return Ok(AtaDrive28 {
                        ata_bus: self,
                        drive_selector: drive,
                    });*/
                }
            } else {
                return Err("ATA Error while running identify");
            }
        } else {
            return Err("Drive does not exist");
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DriveSelector {
    Master,
    Slave,
}
