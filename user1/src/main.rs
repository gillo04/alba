#![no_std]
#![no_main]

use stdlib::fs::*;
use stdlib::graphics::*;
use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    let file = File::load("USER/LOGO    PPM").unwrap();
    let img = Image::new(file, 0, 0).unwrap();
    let mut buffer = [0x0u32; 500 * 500];
    let mut sbuffer = ScreenBuffer::new(0, 0, 500, 500, &mut buffer);
    println!("{} {} {}", img.width, img.height, img.start);
    sbuffer.draw(&img);
    sbuffer.put();
    loop {}
}
