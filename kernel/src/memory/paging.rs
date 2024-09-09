use super::{clear_page, println, MEMORY_MANAGER};

#[repr(C)]
pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    const fn new() -> PageTable {
        PageTable([PageTableEntry::new(); 512])
    }

    // Maps the physical address to the virtual address
    pub fn map(&mut self, paddr: u64, vaddr: u64, depth: u32) {
        let index = ((vaddr >> 12) >> (9 * depth)) & 0x1ff;

        if depth == 0 {
            let mut entry = PageTableEntry::new();
            entry.set_flag(FlagsOffset::Writable, true);
            entry.set_flag(FlagsOffset::Present, true);
            entry.set_physical_address(paddr);

            self.0[index as usize] = entry;
            return;
        }

        if !self.0[index as usize].get_flag(FlagsOffset::Present) {
            let new_table = MEMORY_MANAGER.lock().physical_map.alloc_frame();
            clear_page(new_table);

            let mut entry = PageTableEntry::new();
            entry.set_flag(FlagsOffset::Writable, true);
            entry.set_flag(FlagsOffset::Present, true);
            entry.set_physical_address(new_table);

            self.0[index as usize] = entry;
        }

        unsafe {
            (&mut *(self.0[index as usize].get_physical_address() as *mut PageTable)).map(
                paddr,
                vaddr,
                depth - 1,
            )
        };
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct PageTableEntry(u64);

impl PageTableEntry {
    const fn new() -> PageTableEntry {
        PageTableEntry(0)
    }

    // Getters
    fn get_flag(&self, offset: FlagsOffset) -> bool {
        (self.0 >> offset as u32) & 1 != 0
    }

    fn get_physical_address(&self) -> u64 {
        self.0 & 0xf_ffff_ffff_f000
    }

    // Setters
    fn set_flag(&mut self, offset: FlagsOffset, value: bool) {
        self.0 &= !(1 << offset as u32);
        self.0 |= (value as u64) << (offset as u32);
    }

    fn set_physical_address(&mut self, addr: u64) {
        self.0 &= !0xf_ffff_ffff_f000;
        self.0 |= addr & 0xf_ffff_ffff_f000;
    }
}

#[derive(Clone, Copy)]
enum FlagsOffset {
    Present = 0,
    Writable,
    UserAccessible,
    WriteThroughCaching,
    DisableCache,
    Accessed,
    Dirty,
    HugePage,
    Global,
    NoExecute = 63,
}
