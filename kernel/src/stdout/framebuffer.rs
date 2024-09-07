use super::*;

pub struct FrameBuffer {
    pub base: u64,
    pub width: u64,
    pub height: u64,
    pub pixels_per_scanline: u64,
}

impl FrameBuffer {
    pub const fn new(base: u64, width: u64, height: u64, pixels_per_scanline: u64) -> FrameBuffer {
        FrameBuffer {
            base,
            width,
            height,
            pixels_per_scanline,
        }
    }

    #[inline]
    pub fn set_pixel(&self, x: u64, y: u64, color: Color) {
        unsafe {
            *(self.base as *mut u32).offset((x + y * self.pixels_per_scanline) as isize) = color.0;
        }
    }

    pub fn copy_from(&self, buffer: u64) {
        for i in 0..self.height {
            unsafe {
                asm!(
                    "rep movsd",
                    in("rcx") self.width,
                    in("rsi") buffer + i * self.width * 4,
                    in("rdi") self.base + i * self.pixels_per_scanline * 4,
                );
            }
        }
    }

    pub fn clear(&self, color: Color) {
        unsafe {
            asm!(
                "rep stosd",
                in("rcx") self.height * self.pixels_per_scanline,
                in("eax") color.0,
                in("rdi") self.base,
            );
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Color(pub u32);

impl Color {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color(b as u32 | ((g as u32) << 8) | ((r as u32) << 16))
    }

    pub const fn from_u32(color: u32) -> Color {
        Color(color)
    }
}
