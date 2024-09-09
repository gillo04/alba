use super::Mutex;

pub static TSS: Mutex<Tss> = Mutex::new(Tss::new());

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct SystemSegmentDescriptor(pub u64, pub u64);

impl SystemSegmentDescriptor {
    pub fn new_tss_segment(base: &Tss) -> SystemSegmentDescriptor {
        let limit = size_of::<Tss>() - 1;
        let base = base as *const Tss as u64;
        let low_word = (limit & 0xffff) as u64
            | ((base & 0xffffff) << 16)
            | (0x89 << 40)
            | (((limit as u64 >> 16) & 0xff) << 48)
            | (((base >> 24) & 0xff) << 56);
        let high_word = base >> 32;
        SystemSegmentDescriptor(low_word, high_word)
    }
}

#[repr(C, packed)]
pub struct Tss {
    pub _reserved1: u32,
    pub privilege_stacks: [u64; 3],
    pub _reserved2: u64,
    pub interrupt_stacks: [u64; 7],
    pub _reserved3: u64,
    pub _reserved4: u16,
    pub iopb: u16,
}

impl Tss {
    pub const fn new() -> Tss {
        Tss {
            _reserved1: 0,
            privilege_stacks: [0; 3],
            _reserved2: 0,
            interrupt_stacks: [0; 7],
            _reserved3: 0,
            _reserved4: 0,
            iopb: 0,
        }
    }
}
