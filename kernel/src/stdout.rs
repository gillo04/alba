mod font;
mod framebuffer;

use super::STDOUT;
use crate::uefi::*;
use core::arch::asm;
use core::ffi::c_void;
use font::*;
use framebuffer::*;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::stdout::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    STDOUT.lock().write_fmt(args).unwrap();
}

// Initializes the STDOUT global variable. Depends on correct configuration of the SYSTEM_TABLE
pub fn init(system_table: *const SystemTable, buffer: Option<u64>) -> Result<(), ()> {
    let mut gop = 0 as *const GraphicsOutputProtocol;
    unsafe {
        ((*(*system_table).boot_services).locate_protocol)(
            &GOP_GUID as *const Guid,
            0 as *const c_void,
            &mut gop as *mut *const GraphicsOutputProtocol as *mut *const c_void,
        )
    };

    unsafe {
        *STDOUT.lock() = StdOut::new(
            (*(*gop).mode).frame_buffer_base as u64,
            (*(*(*gop).mode).info).horizontal_resolution as u64,
            (*(*(*gop).mode).info).vertical_resolution as u64,
            (*(*(*gop).mode).info).pixels_per_scan_line as u64,
            buffer,
        );
    }

    // Clear screen
    STDOUT.lock().frame_buffer.clear(Color::from_rgb(0, 0, 0));

    Ok(())
}

pub struct StdOut {
    pub frame_buffer: FrameBuffer,
    col: u64,
    row: u64,
    color: Color,
    pub buffer: Option<u64>,
}

impl StdOut {
    pub const fn new(
        base: u64,
        width: u64,
        height: u64,
        pixels_per_scanline: u64,
        buffer: Option<u64>,
    ) -> StdOut {
        StdOut {
            frame_buffer: FrameBuffer::new(base, width, height, pixels_per_scanline),
            col: 0,
            row: 0,
            color: Color::from_u32(0xb0b0b0),
            buffer,
        }
    }

    fn print_string(&mut self, s: &str) {
        if let Some(_buffer) = self.buffer {
            // TODO
        } else {
            let mut iter = s.bytes();
            loop {
                let c = iter.next();
                if let Some(c) = c {
                    if c == '\\' as u8 {
                        let d = iter.next().expect("Missing color letter");
                        self.color = match d as char {
                            'w' => Color::from_u32(0xb0b0b0),
                            'r' => Color::from_u32(0xff0000),
                            'g' => Color::from_u32(0x00ff00),
                            'b' => Color::from_u32(0x0000ff),
                            _ => panic!("Unsupported color"),
                        };
                    } else if c == '\n' as u8 {
                        self.row += 1;
                        self.col = 0;
                    } else if c == '\t' as u8 {
                        self.col += 4 - (self.col % 4);
                    } else {
                        for i in 0..GLYPH_HEIGHT {
                            let row: u16 = PSF[c as usize * 32 + i as usize];
                            for j in 0..GLYPH_WIDTH {
                                let color = if (row >> (GLYPH_WIDTH - j - 1)) & 1 == 1 {
                                    self.color
                                } else {
                                    Color::from_u32(0)
                                };
                                self.frame_buffer.set_pixel(
                                    self.col * GLYPH_WIDTH + j,
                                    self.row * GLYPH_HEIGHT + i,
                                    color,
                                );
                            }
                        }
                        self.col += 1;
                    }

                    // Bounds checking
                    if self.col * GLYPH_WIDTH >= self.frame_buffer.width - GLYPH_WIDTH {
                        self.col = 0;
                        self.row += 1;
                    }

                    if self.row * GLYPH_HEIGHT >= self.frame_buffer.height - GLYPH_HEIGHT {
                        self.row -= 1;
                        unsafe {
                            core::ptr::copy_nonoverlapping(
                                (self.frame_buffer.base
                                    + self.frame_buffer.pixels_per_scanline * GLYPH_HEIGHT * 4)
                                    as *const u32,
                                self.frame_buffer.base as *mut u32,
                                (self.frame_buffer.pixels_per_scanline
                                    * (self.frame_buffer.height - GLYPH_HEIGHT))
                                    as usize,
                            );
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }
}

impl core::fmt::Write for StdOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print_string(s);
        Ok(())
    }
}
