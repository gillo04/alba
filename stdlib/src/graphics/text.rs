use super::*;
use alloc::string::*;

pub struct Font {
    file: File,
    byte_size: u8,
}

impl Font {
    pub fn new(file: File) -> Result<Font, &'static str> {
        if unsafe { *(file.ptr as *const u16) } != 0x0436 {
            return Err("Unsupported psf format");
        }

        // Flags
        let flags = unsafe { *(file.ptr as *const u8).offset(2) };
        if flags & 0b10 == 0 {
            return Err("Unsupported psf1 format");
        }

        let byte_size = unsafe { *(file.ptr as *const u8).offset(3) };

        Ok(Font { file, byte_size })
    }

    pub fn draw_char(
        &self,
        char: char,
        x: i64,
        y: i64,
        scale: u64,
        color: u32,
        sb: &mut ScreenBuffer,
    ) {
        let off = self.file.ptr + 4 + char as u64 * self.byte_size as u64;
        for i in 0..(self.byte_size as u64 * scale) {
            let b = unsafe { *(off as *const u8).offset(i as isize / scale as isize) };
            for j in 0..8 * scale {
                if (b >> (8 - (j / scale) - 1)) & 1 == 1 {
                    sb.base[((y + i as i64) * sb.w as i64 + x + j as i64) as usize] = color;
                }
            }
        }
    }

    pub fn draw_string(
        &self,
        s: String,
        x: i64,
        y: i64,
        scale: u64,
        color: u32,
        sb: &mut ScreenBuffer,
    ) {
        let mut x1 = x;
        let mut y1 = y;
        for (i, c) in s.chars().enumerate() {
            if c == '\n' {
                x1 = x;
                y1 += self.byte_size as i64;
            } else {
                self.draw_char(c, x1, y1, scale, color, sb);
                x1 += 8 * scale as i64;
            }
        }
    }
}
