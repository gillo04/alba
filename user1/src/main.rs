#![no_std]
#![no_main]

use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    let mut buffer = [0x0u32; 255 * 255];
    let sbuffer = ScreenBuffer::new(0, 0, 255, 255, &buffer as *const u32 as u64);
    let mut direction: i64 = 1;

    let mut rect = Rectangle {
        x: -255 / 2,
        y: -255 / 2,
        w: 255,
        h: 255,
        color: 0xff0000,
    };
    loop {
        rect.x += direction;
        rect.y += direction;
        if rect.x >= 255 || rect.x <= -128 {
            direction = -direction;
        }
        sbuffer.clear(0);
        sbuffer.draw(&rect);
        sbuffer.put();
    }
}
