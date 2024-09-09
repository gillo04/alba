#[allow(unused)]
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

#[inline]
pub fn inb(port: u16) -> u8 {
    let mut result: u8;
    unsafe {
        asm!("in al, dx", out("al") result, in("dx") port);
    }
    result
}

#[inline]
pub fn outb(port: u16, data: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") data);
    }
}

#[inline]
pub fn inw(port: u16) -> u16 {
    let mut result: u16;
    unsafe {
        asm!("in ax, dx", out("ax") result, in("dx") port);
    }
    result
}

#[inline]
pub fn outw(port: u16, data: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") data);
    }
}

#[inline]
pub fn ind(port: u16) -> u32 {
    let mut result: u32;
    unsafe {
        asm!("in eax, dx", out("eax") result, in("dx") port);
    }
    result
}

#[inline]
pub fn outd(port: u16, data: u32) {
    unsafe {
        asm!("out dx, eax", in("dx") port, in("eax") data);
    }
}

#[inline]
pub fn wait_io() {
    outb(0x80, 0);
}
