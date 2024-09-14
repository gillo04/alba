#![no_std]
#![no_main]

use stdlib::alloc::vec::*;
use stdlib::alloc::*;
use stdlib::fs::*;
use stdlib::graphics::gui::draw_gui_tree;
use stdlib::graphics::gui::*;
use stdlib::graphics::*;
use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    stdlib::heap::init().unwrap();

    let mut buffer = vec![0x0u32; 500 * 500];
    let mut sbuffer = ScreenBuffer::new(0, 0, 500, 500, &mut buffer[..]);

    let x1 = 100;
    let y1 = 100;
    let x2 = 200;
    let y2 = 50;

    let c1 = Circle {
        x: x1,
        y: y1,
        r: 10,
        color: 0xff0000,
    };
    let c2 = Circle {
        x: x2,
        y: y2,
        r: 10,
        color: 0xff0000,
    };
    let line = Line {
        x1,
        y1,
        x2,
        y2,
        color: 0xffffff,
    };

    sbuffer.clear(0);
    c1.draw(&mut sbuffer);
    c2.draw(&mut sbuffer);
    line.draw(&mut sbuffer);
    sbuffer.put();
    loop {}
    let mut prev_time = get_milliseconds_since_startup();
    loop {
        sbuffer.clear(0);
        sbuffer.put();
        while get_milliseconds_since_startup() - prev_time < 6 {}
        prev_time = get_milliseconds_since_startup();
    }
}
