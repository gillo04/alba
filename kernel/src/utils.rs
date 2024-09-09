use core::arch::asm;

#[inline]
pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

pub fn clear_page(page: u64) {
    for i in 0..4096 / 8 {
        unsafe {
            *(page as *mut u64).offset(i) = 0;
        }
    }
}
