#![no_std]
#![no_main]

use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    let mut buffer = [0xff0000u32; 255 * 255];
    for i in 0..255 {
        for j in 0..255 {
            buffer[i * 255 + j] = ((i << 16) | ((255 - j) << 8) | j) as u32;
        }
    }
    let sbuffer = ScreenBuffer::new(100, 200, 255, 255, &buffer as *const u32 as u64);
    sbuffer.put();
    loop {}
}
