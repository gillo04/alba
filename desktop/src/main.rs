#![no_std]
#![no_main]

use stdlib::alloc::string::*;
use stdlib::alloc::vec::*;
use stdlib::alloc::*;
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

    let pointer = Image::new(File::load("USER/POINTER PPM").unwrap()).unwrap();
    let screen_size = get_screen();

    let font = Font::new(File::load("USER/FONT    PSF").unwrap()).unwrap();

    let mut buffer = vec![0x0u32; (screen_size.width * screen_size.height) as usize];
    let mut sbuffer =
        ScreenBuffer::new(0, 0, screen_size.width, screen_size.height, &mut buffer[..]);

    let mut file_name = String::new();
    loop {
        let key_pressed = get_key();
        if let Some((char, scancode)) = key_pressed {
            if scancode == 0x1c {
                exec(file_name.as_str()).unwrap();
                file_name.clear();
            } else if scancode == 0x0e && file_name.len() > 0 {
                file_name.pop();
            } else if char != 0 {
                file_name.push(char as char);
            }
        }
        let mouse_pos = get_mouse_position();
        sbuffer.clear(0x72A0C1);

        font.draw_string(
            format!("What executable do you want to load?: {}", file_name),
            100,
            200,
            2,
            0x0,
            &mut sbuffer,
        );

        pointer.draw(
            &mut sbuffer,
            mouse_pos.0 as i64,
            mouse_pos.1 as i64,
            100,
            100,
        );
        sbuffer.put();
    }
}
