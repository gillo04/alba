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

const TAB_HEIGHT: u64 = 40;

struct Tab {
    color: u32,
    pid: u32,
}

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    stdlib::heap::init().unwrap();
    let smh = stdlib::desktop::server_init();

    let pointer = Image::new(File::load("USER/POINTER PPM").unwrap()).unwrap();
    let screen_size = get_screen();

    let font = Font::new(File::load("USER/FONT    PSF").unwrap()).unwrap();

    let mut buffer = vec![0x0u32; (screen_size.width * screen_size.height) as usize];
    let mut sbuffer =
        ScreenBuffer::new(0, 0, screen_size.width, screen_size.height, &mut buffer[..]);

    // Create file icons on the desktop
    let file_icon = Image::new(File::load("USER/EXE_ICONPPM").unwrap()).unwrap();
    let files = vec![
        ("USER/GUI_DEMO", &file_icon),
        ("USER/USER2", &file_icon),
        // ("USER/USER1", &file_icon),
    ];

    let mut tabs: Vec<Tab> = vec![];
    let mut drag_anchor: Option<(u64, u64)> = None;
    let mut current_drag: usize = 0;
    let mut is_left_pressed: bool = false;

    let mut prev_pid = 0;
    loop {
        let mouse_pos = get_mouse();

        // Check for acknowledgements and advance free pointer
        if smh.ack == 1 {
            // Advance free space ptr
            smh.advance_free_space();

            // Create tab
            tabs.push(Tab {
                color: 0xcccccc,
                pid: prev_pid,
            });
        }

        // Draw
        sbuffer.clear(0x72A0C1);

        // Draw windows
        for (i, window) in smh.iter().enumerate() {
            if i >= tabs.len() {
                break;
            }

            // Bounds checking
            let left = u64::clamp(window.x, 0, sbuffer.w);
            let top = u64::clamp(window.y, 0, sbuffer.h);
            let right = u64::clamp(window.x + window.width, 0, sbuffer.w);
            let bottom = u64::clamp(window.y + window.height, 0, sbuffer.h);

            let window_base = &window.data as *const () as u64 as *const u32;
            for i in top..bottom {
                for j in left..right {
                    unsafe {
                        sbuffer.base[(i * sbuffer.w + j) as usize] = *window_base
                            .offset(((i - window.y) * window.width + (j - window.x)) as isize);
                    }
                }
            }

            // Draw tab
            let tab = Rectangle {
                rect: Rect {
                    x: window.x as i64,
                    y: (window.y - TAB_HEIGHT) as i64,
                    width: window.width,
                    height: TAB_HEIGHT,
                },
                color: tabs[i].color,
            };
            tab.draw(&mut sbuffer);
        }

        // Check if mouse is in tab
        if mouse_pos.2 {
            let mx = mouse_pos.0;
            let my = mouse_pos.1;
            if !is_left_pressed {
                for (i, window) in smh.iter().enumerate() {
                    if i >= tabs.len() {
                        break;
                    }

                    let tab = Rectangle {
                        rect: Rect {
                            x: window.x as i64,
                            y: (window.y - TAB_HEIGHT) as i64,
                            width: window.width,
                            height: TAB_HEIGHT,
                        },
                        color: tabs[i].color,
                    };
                    if tab.rect.point_intersection(mx as i64, my as i64) {
                        drag_anchor = Some((mx - tab.rect.x as u64, my - tab.rect.y as u64));
                        current_drag = i;
                    }
                }
            } else if drag_anchor.is_some() {
                let fw = smh.iter_mut().enumerate().nth(current_drag);
                if let Some((i, window)) = fw {
                    let drag_anchor = drag_anchor.unwrap();
                    window.x = mouse_pos.0 - drag_anchor.0;
                    window.y = mouse_pos.1 - drag_anchor.1 + TAB_HEIGHT;
                }
            }

            // Check exe click
            if !is_left_pressed {
                for (i, file) in files.iter().enumerate() {
                    let r = Rect {
                        x: i as i64 * 150,
                        y: 0,
                        width: 100,
                        height: 100,
                    };

                    if r.point_intersection(mx as i64, my as i64) {
                        prev_pid = exec(file.0).unwrap();
                    }
                }
            }
            is_left_pressed = true;
        } else {
            drag_anchor = None;
            is_left_pressed = false;
        }

        // Draw file icons
        for (i, file) in files.iter().enumerate() {
            file.1.draw(&mut sbuffer, i as i64 * 150, 0, 100, 100);

            font.draw_string(
                &String::from(file.0),
                i as i64 * 150,
                100,
                1,
                0x0,
                &mut sbuffer,
            );
        }

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
