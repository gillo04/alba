use super::*;

pub fn server_init() -> &'static mut SharedMemoryHeader {
    let shared_memory = get_shared_page();
    let sm = SharedMemoryHeader {
        free_space_offset: shared_memory + 8 * 2,
        ack: 0,
    };
    unsafe {
        *(shared_memory as *mut SharedMemoryHeader) = sm;
        &mut *(shared_memory as *mut SharedMemoryHeader)
    }
}

pub fn client_init(window: &WindowHeader) -> (&'static WindowHeader, ScreenBuffer) {
    let smh = unsafe { &mut *(get_shared_page() as *mut SharedMemoryHeader) };

    // Set shared window variables
    unsafe {
        *(smh.free_space_offset as *mut WindowHeader) = *window;
    }

    // Acknowledge
    smh.ack = 1;

    let out_win = unsafe { &*(smh.free_space_offset as *const WindowHeader) };
    let out_sb = unsafe {
        let slice =
            core::slice::from_raw_parts_mut(&out_win.data as *const () as *mut u32, 500 * 500);
        ScreenBuffer::new(0, 0, window.width, window.height, &mut slice[..])
    };

    (out_win, out_sb)
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SharedMemoryHeader {
    pub free_space_offset: u64,
    pub ack: u64,
}

impl SharedMemoryHeader {
    pub fn iter(&self) -> WindowIterator {
        WindowIterator {
            next_window: (self as *const SharedMemoryHeader as u64)
                + size_of::<SharedMemoryHeader>() as u64,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct WindowHeader {
    pub width: u64,
    pub height: u64,
    pub x: u64,
    pub y: u64,
    pub data: (),
}

pub struct WindowIterator {
    next_window: u64,
}

impl core::iter::Iterator for WindowIterator {
    type Item = &'static WindowHeader;
    fn next(&mut self) -> Option<Self::Item> {
        let header = unsafe { &*(self.next_window as *const WindowHeader) };
        if header.width == 0 && header.height == 0 {
            return None;
        }

        let out = Some(header);
        self.next_window += size_of::<WindowHeader>() as u64 + header.width * header.height * 4;
        out
    }
}
