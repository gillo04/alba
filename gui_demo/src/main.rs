#![no_std]
#![no_main]

use stdlib::alloc::string::*;
use stdlib::alloc::vec::*;
use stdlib::alloc::*;
use stdlib::desktop::*;
use stdlib::fs::*;
use stdlib::graphics::gui::draw_gui_tree;
use stdlib::graphics::gui::*;
use stdlib::graphics::text::*;
use stdlib::graphics::*;
use stdlib::*;

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    stdlib::heap::init().unwrap();

    // Initialize desktop window
    let window = WindowHeader {
        width: 500,
        height: 500,
        x: 100,
        y: 200,
        data: (),
    };
    let (window, mut window_buffer) = stdlib::desktop::client_init(&window);

    // Initialize buffer
    let mut buffer = vec![0x0u32; 500 * 500];
    let mut sbuffer = ScreenBuffer::new(0, 0, 500, 500, &mut buffer[..]);

    // Load assets
    let f = File::load("USER/LOGO    PPM").unwrap();
    let img = Image::new(f).unwrap();
    let font = Font::new(File::load("USER/FONT    PSF").unwrap()).unwrap();

    // Build GUI tree
    let mut image = GuiRect {
        width: Dimension::Percentage(1.0),
        height: Dimension::Relative,
        margin_y: 5,
        fill: Fill::Image(&img),
        ..Default::default()
    };
    let slab = GuiRect {
        width: Dimension::Percentage(1.0),
        height: Dimension::Absolute(60),
        margin_y: 5,
        fill: Fill::Solid(0xff7777),
        fill_active: Some(Fill::Solid(0xff0000)),
        text: Some((String::from("Lorem ipsum\ndolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat."), &font)),
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
        children: vec![image.clone(), slab.clone()],
        ..Default::default()
    };

    let mut gui_root = GuiRect {
        width: Dimension::Absolute(500),
        height: Dimension::Absolute(500),
        padding_x: 5,
        padding_y: 5,
        layout: Layout::Horizontal,
        fill: Fill::Solid(0xffffff),
        children: vec![card.clone() /*card.clone()*/],
        ..Default::default()
    };

    let mut direction: i64 = 1;
    let mut width = 250;
    loop {
        width += direction;
        if width >= 500 || width <= 100 {
            direction = -direction;
        }
        gui_root.width = Dimension::Absolute(width as u64);

        let mouse = get_mouse();
        let io = IoState {
            mouse_pos: (mouse.0 - window.x, mouse.1 - window.y),
            left_button: mouse.2,
            right_button: mouse.3,
        };
        draw_gui_tree(&gui_root, &mut sbuffer, &io);
        window_buffer.copy_from_screen_buffer(&sbuffer);
    }
}
