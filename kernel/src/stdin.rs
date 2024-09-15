#![allow(unused)]

pub mod scancodes;

use super::{print, stdout, Mutex, String};
use core::arch::asm;
use scancodes::*;

pub static STDIN: Mutex<Stdin> = Mutex::new(Stdin::new());

// It's very ugly and is at risk of a race condition with the keyboard ISR, but it's good enough
// for now
pub fn stdin() -> String {
    let mut out = String::new();

    loop {
        let int = STDIN.lock().keyboard_int;
        if let Some(scancode) = int {
            // Caps lock
            if scancode == 0x3a {
                let cl = STDIN.lock().is_caps_lock_active;
                STDIN.lock().is_caps_lock_active = !cl;
            }

            // Backspace
            if scancode == 0x0e && out.len() > 0 {
                out.pop();
                stdout::STDOUT.lock().backspace();
            }

            if SCAN_CODE_SET1[scancode as usize] != '\0' {
                let mut char = SCAN_CODE_SET1[scancode as usize];

                if char >= 'A' && char <= 'Z' {
                    let cl = STDIN.lock().is_caps_lock_active;
                    let mut s = STDIN.lock().pressed_scancodes[0x2a];
                    s |= STDIN.lock().pressed_scancodes[0x36];
                    if !(cl ^ s) {
                        char = char.to_ascii_lowercase();
                    }
                }
                print!("{}", char);
                out.push(char);
            }
            STDIN.lock().keyboard_int = None;

            // Enter
            if scancode == 0x1c {
                break;
            }
        }

        unsafe {
            asm!("hlt");
        }
    }

    out
}

pub struct Stdin {
    pub pressed_scancodes: [bool; 256],
    pub keyboard_int: Option<u8>,
    pub is_shift_pressed: bool,
    pub is_caps_lock_active: bool,
}

impl Stdin {
    const fn new() -> Stdin {
        Stdin {
            pressed_scancodes: [false; 256],
            keyboard_int: None,
            is_shift_pressed: false,
            is_caps_lock_active: false,
        }
    }
}
