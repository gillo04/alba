#![allow(unused)]

use super::{print, println};
use crate::memory::*;
use crate::utils::*;
use alloc::vec::*;

pub struct ElfExecutable {
    mapping: VirtualMapping,
}

impl ElfExecutable {
    pub fn new(mapping: VirtualMapping) -> ElfExecutable {
        ElfExecutable { mapping }
    }

    #[inline]
    fn get_header(&self) -> &ElfHeader64 {
        unsafe { &*(self.mapping.vaddr as *const ElfHeader64) }
    }

    fn get_sections(&self) -> &[ElfSectionHeaderEntry64] {
        let header = self.get_header();

        unsafe {
            core::slice::from_raw_parts(
                (header as *const ElfHeader64 as u64 + header.section_header_offset)
                    as *const ElfSectionHeaderEntry64,
                header.section_header_entry_count as usize,
            )
        }
    }

    fn get_segments(&self) -> &[ElfProgramHeaderEntry64] {
        let header = self.get_header();
        unsafe {
            core::slice::from_raw_parts(
                (header as *const ElfHeader64 as u64 + header.program_header_offset)
                    as *const ElfProgramHeaderEntry64,
                header.program_header_entry_count as usize,
            )
        }
    }

    // Load section at an arbitrary virtual address
    fn load_section(&self, section: u64) -> VirtualMapping {
        let section = &self.get_sections()[section as usize];
        let file_offset = section.file_offset;
        let file_size = section.file_size;

        let out = KERNEL_VALLOCATOR.lock().alloc_pages(file_size / 0x1000 + 1);
        for i in 0..file_size {
            unsafe {
                *(out.vaddr as *mut u8).offset(i as isize) =
                    *(self.mapping.vaddr as *const u8).offset(file_offset as isize + i as isize);
            }
        }
        out
    }

    // Loads segment at the correct virtual address and refreshes cr3
    fn load_segment(&self, segment: u64) -> VirtualMapping {
        let segment = &self.get_segments()[segment as usize];
        let file_offset = segment.file_offset;
        let file_size = segment.file_size;

        // Create memory mapping
        let vaddr = segment.virtual_address;
        let memory_size = segment.memory_size;

        let page_count = (memory_size + vaddr & 0xfff) as usize / 0x1000 + 1;
        let mut out = VirtualMapping::new(vaddr & !0xfff, Vec::with_capacity(page_count));
        for i in 0..page_count {
            out.frames
                .push(MEMORY_MANAGER.lock().physical_map.alloc_frame());
            clear_page(out.frames[i as usize]);
        }
        let plm4 = MEMORY_MANAGER.lock().get_plm4();
        plm4.map_mapping(&out);
        MEMORY_MANAGER.lock().set_plm4(plm4);

        for i in 0..page_count {
            clear_page(out.frames[i as usize]);
        }

        // Copy segment
        for i in 0..file_size {
            unsafe {
                *(vaddr as *mut u8).offset(i as isize) =
                    *(self.mapping.vaddr as *const u8).offset(file_offset as isize + i as isize);
            }
        }
        out
    }

    pub fn list_sections(&self) {
        let header = self.get_header();
        let sections = self.get_sections();

        // Load section strings sector
        let strings = (self.mapping.vaddr
            + sections[header.section_name_strings_index as usize].file_offset)
            as *const u8;

        println!("Sections: ");
        for section in sections {
            let mut i = section.section_name_offset;
            let mut string_len = 0;
            while unsafe { *strings.offset(i as isize) != 0 } {
                i += 1;
                string_len += 1;
            }

            let string = unsafe {
                core::str::from_raw_parts(
                    strings.offset(section.section_name_offset as isize),
                    string_len,
                )
            };
            println!("\t{}", string);
        }
    }

    pub fn list_segments(&self) {
        let header = self.get_header();
        let segments = self.get_segments();

        let x = header.program_header_entry_count as usize;
        println!("ok {}", x);
        for segment in segments {
            let t = segment.segment_type;
            let f = segment.flags;

            let file_o = segment.file_offset;
            let virt_o = segment.virtual_address;
            let file_s = segment.file_size;
            let mem_s = segment.memory_size;
            print!(
                "{:>10?}\t{:06x}-{:06x}\t{:06x}-{:06x}\t",
                t, file_o, virt_o, file_s, mem_s
            );
            if f & ElfSegmentFlags::Executable as u32 != 0 {
                print!("Executable ");
            }
            if f & ElfSegmentFlags::Readable as u32 != 0 {
                print!("Readable ");
            }
            if f & ElfSegmentFlags::Writeable as u32 != 0 {
                print!("Writeable ");
            }
            println!("");
        }
    }

    pub fn load_all(&self) -> Vec<VirtualMapping> {
        let mut out = Vec::new();
        let segments = self.get_segments();

        for (i, segment) in segments.iter().enumerate() {
            let t = segment.segment_type;
            if t == ElfSegmentType::Load {
                out.push(self.load_segment(i as u64));
            }
        }

        out
    }

    pub fn get_entry(&self) -> u64 {
        self.get_header().entry_point
    }
}

#[repr(C, packed)]
struct ElfHeader64 {
    magic: [u8; 4],
    address_size: u8,
    endianness: u8,
    version1: u8,
    abi: ElfAbi,
    abi_version: u8,
    _reserved: [u8; 7],
    file_type: ElfFileType,
    instruction_set: u16,
    version2: u32,
    entry_point: u64,
    program_header_offset: u64,
    section_header_offset: u64,
    flags: u32,
    elf_header_size: u16,
    program_header_entry_size: u16,
    program_header_entry_count: u16,
    section_header_entry_size: u16,
    section_header_entry_count: u16,
    section_name_strings_index: u16,
}

#[repr(C, packed)]
struct ElfProgramHeaderEntry64 {
    segment_type: ElfSegmentType,
    flags: u32,
    file_offset: u64,
    virtual_address: u64,
    physical_address: u64,
    file_size: u64,
    memory_size: u64,
    align: u64,
}

#[repr(C, packed)]
struct ElfSectionHeaderEntry64 {
    section_name_offset: u32,
    section_type: u32,
    section_flags: u64,
    virtual_address: u64,
    file_offset: u64,
    file_size: u64,
    link_section: u32,
    section_info: u32,
    section_align: u64,
    entry_size: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum ElfAbi {
    SystemV = 0x00,
    HpUx = 0x01,
    NetBdd = 0x02,
    Linux = 0x03,
    GnuHurd = 0x04,
    Solaris = 0x06,
    Aix = 0x07,
    Irix = 0x08,
    FreeBsd = 0x09,
    Tru64 = 0x0A,
    NovellModesto = 0x0B,
    OpenBsd = 0x0C,
    OpenVms = 0x0D,
    NonStopKernel = 0x0E,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
enum ElfFileType {
    None = 0x00,
    Rel = 0x01,
    Exec = 0x02,
    Dyn = 0x03,
    Core = 0x04,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
enum ElfSegmentType {
    Null = 0x00000000,
    Load = 0x00000001,
    Dynamic = 0x00000002,
    InterpreterInfo = 0x00000003,
    Note = 0x00000004,
    Reserved = 0x00000005,
    ProgramHeader = 0x00000006,
    Tls = 0x00000007,
}

enum ElfSegmentFlags {
    Executable = 0x1,
    Writeable = 0x2,
    Readable = 0x4,
}
