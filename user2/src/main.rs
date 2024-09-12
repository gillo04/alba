#![no_std]
#![no_main]

use stdlib::graphics::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    let mut buffer = [0x0u32; 255 * 255];
    let mut sbuffer = ScreenBuffer::new(500, 0, 255, 255, &mut buffer);
    let mut direction: i64 = 1;

    let mut circ = Circle {
        x: -256,
        y: -256,
        r: 255 / 2,
        color: 0x00ff00,
    };

    loop {
        circ.x += direction;
        circ.y += direction;
        if circ.x >= 256 + 128 || circ.x <= -256 {
            direction = -direction;
        }
        sbuffer.clear(0);
        sbuffer.draw(&circ);
        sbuffer.put();
    }
}
