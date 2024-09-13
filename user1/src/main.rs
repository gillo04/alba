#![no_std]
#![no_main]

use stdlib::fs::*;
use stdlib::graphics::*;
use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    /*let file = File::load("USER/LOGO    PPM").unwrap();
    let img = Image::new(file, 0, 0).unwrap();*/
    let mut buffer = [0x0u32; 500 * 500];
    let mut sbuffer = ScreenBuffer::new(0, 0, 500, 500, &mut buffer);
    let mut rect = Rectangle {
        x: 10,
        y: 10,
        w: 100,
        h: 100,
        fill: Fill::Gradient(0x00ff00, 0xff0000),
    };
    let mut direction: i32 = 1;
    let mut val: i32 = 0;
    loop {
        rect.fill = Fill::Gradient((255 - val as u32) << 8, (val as u32) << 16);
        val += direction;
        if val == 255 || val == 0 {
            direction = -direction;
        }
        sbuffer.draw(&rect);
        sbuffer.put();
    }
}
