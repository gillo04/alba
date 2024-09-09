#![allow(unused)]

use super::{print, Mutex};

static STDIN: Mutex<Stdin> = Mutex::new(Stdin::new());

struct Stdin {
    pressed_scancodes: [bool; 256],
    keyboard_int: Option<u8>,
}

impl Stdin {
    const fn new() -> Stdin {
        Stdin {
            pressed_scancodes: [false; 256],
            keyboard_int: None,
        }
    }
}
