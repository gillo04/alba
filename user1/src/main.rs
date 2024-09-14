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

    let img = Image::new(File::load("USER/LOGO    PPM").unwrap()).unwrap();

    let mut buffer = vec![0x0u32; 500 * 500];
    let mut sbuffer = ScreenBuffer::new(0, 0, 500, 500, &mut buffer[..]);
    // Build GUI tree
    let slab = GuiRect {
        width: Dimension::Percentage(1.0),
        height: Dimension::Relative,
        margin_y: 5,
        fill: Fill::Image(&img),
        ..Default::default()
    };
    let card = GuiRect {
        width: Dimension::Percentage(0.5),
        height: Dimension::Absolute(300),
        fill: Fill::Solid(0xbbbbbb),
        margin_x: 5,
        margin_y: 5,
        padding_x: 10,
        padding_y: 5,
        children: vec![slab.clone(), slab.clone(), slab.clone()],
        ..Default::default()
    };

    let mut gui_root = GuiRect {
        width: Dimension::Absolute(500),
        height: Dimension::Absolute(500),
        padding_x: 5,
        padding_y: 5,
        layout: Layout::Horizontal,
        fill: Fill::Solid(0xffffff),
        children: vec![card.clone(), card.clone()],
        ..Default::default()
    };

    println!("\n\n\n");
    let mut direction = 1;
    let mut width = 250;
    let mut prev_time = get_milliseconds_since_startup();
    loop {
        width += direction;
        if width >= 500 || width <= 100 {
            direction = -direction;
        }
        gui_root.width = Dimension::Absolute(width as u64);

        draw_gui_tree(&gui_root, &mut sbuffer);
        sbuffer.put();
        while get_milliseconds_since_startup() - prev_time < 6 {}
        prev_time = get_milliseconds_since_startup();
    }
}
