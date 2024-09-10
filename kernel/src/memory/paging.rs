use super::{clear_page, println, VirtualMapping, MEMORY_MANAGER};

const PAGING_BASE: u64 = 0x2222_0000_0000;

#[repr(C)]
pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    const fn new() -> PageTable {
        PageTable([PageTableEntry::new(); 512])
    }

    // Makses space for page tables. Can be called only on plm4
    /*pub fn init(&mut self) {
        let index4 = ((PAGING_BASE >> 12) >> (9 * 3)) & 0x1ff;

        let index3 = ((PAGING_BASE >> 12) >> (9 * 2)) & 0x1ff;
        let frame3 = MEMORY_MANAGER.lock().physical_map.alloc_frame();
        clear_page(frame3);

        let index2 = ((PAGING_BASE >> 12) >> (9 * 1)) & 0x1ff;
        let frame2 = MEMORY_MANAGER.lock().physical_map.alloc_frame();
        clear_page(frame2);

        let index1 = ((PAGING_BASE >> 12) >> (9 * 0)) & 0x1ff;
        let frame1 = MEMORY_MANAGER.lock().physical_map.alloc_frame();
        clear_page(frame1);

        // Build table
        let mut entry = PageTableEntry::new();
        entry.set_flag(FlagsOffset::Writable, true);
        entry.set_flag(FlagsOffset::Present, true);

        unsafe {
            entry.set_physical_address(frame3);
            self.0[index4 as usize] = entry;

            entry.set_physical_address(frame2);
            (&mut *(frame3 as *mut PageTable)).0[index3 as usize] = entry;

            entry.set_physical_address(frame1);
            (&mut *(frame2 as *mut PageTable)).0[index2 as usize] = entry;
        }

        self.map(frame3, frame3 + PAGING_BASE, 3);
        self.map(frame2, frame2 + PAGING_BASE, 3);
        self.map(frame1, frame1 + PAGING_BASE, 3);
    }*/

    // Maps the physical address to the virtual address. Can be called only on plm4
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

    pub fn map_mapping(&mut self, mapping: &VirtualMapping) {
        for (i, frame) in mapping.frames.iter().enumerate() {
            self.map(*frame, mapping.vaddr + i as u64 * 0x1000, 3);
        }
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
