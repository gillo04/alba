#![no_std]
#![no_main]

use stdlib::alloc::vec;
use stdlib::alloc::vec::*;
use stdlib::fs::*;
use stdlib::graphics::*;
use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    stdlib::heap::init().unwrap();

    println!("Hello, user world!");

    let mut buffer: Vec<u32> = vec![0; 100 * 150];
    let mut sbuffer = ScreenBuffer::new(10, 20, 100, 150, &mut buffer);
    sbuffer.clear(0xffff00);
    sbuffer.put();

    let screen = get_screen();
    println!("Screen: {}x{}", screen.width, screen.height);

    print!("Enter key: ");
    loop {
        let key = get_key();
        if let Some((c, _)) = key {
            println!("{}", c as char);
            break;
        }
    }

    print!("Mouse: ");
    loop {
        let mouse = get_mouse();
        if mouse.2 {
            println!("left");
            break;
        }
    }

    let img = Image::new(File::load("USER/LOGO    PPM").unwrap()).unwrap();
    img.draw(&mut sbuffer, 0, 0, 100, 100);
    sbuffer.put();

    let ms = get_milliseconds_since_startup();
    println!("Ms: {}", ms);
    loop {}
}
