use core::arch::asm;

#[inline]
pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
