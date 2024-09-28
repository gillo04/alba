#![allow(unused)]

pub mod gui;
pub mod text;

use super::*;

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
}

impl Rect {
    pub fn point_intersection(&self, px: i64, py: i64) -> bool {
        px >= self.x
            && px < self.x + self.width as i64
            && py >= self.y
            && py < self.y + self.height as i64
    }
}

pub struct Line {
    pub x1: i64,
    pub y1: i64,
    pub x2: i64,
    pub y2: i64,
    pub color: u32,
}

impl Line {
    pub fn draw(&self, sb: &mut ScreenBuffer) {
        let mut dx = self.x2 - self.x1;
        let mut dy = self.y2 - self.y1;
        if dy == 0 {
            dy = 1;
        }
        if dx == 0 {
            dx = 1;
        }

        /*let mut x1 = i64::min(self.x1, self.x2);
        let mut x2 = i64::max(self.x1, self.x2);
        let mut y1 = i64::min(self.y1, self.y2);
        let mut y2 = i64::max(self.y1, self.y2);*/

        let mut step_x = 0;
        let mut step_y = 0;
        let mut step_count = 0;
        if dx.abs() > dy.abs() {
            step_x = dx / dy.abs();
            step_y = 1 * dy.signum();
            step_count = dy.abs();
        } else {
            step_x = 1 * dx.signum();
            step_y = dy / dx.abs();
            step_count = dx.abs();
        }

        let mut x = self.x1;
        let mut y = self.y1;
        while step_count > 0 {
            for i in 0..step_x.abs() {
                sb.base[(y * sb.w as i64 + x + i * dx.signum()) as usize] = self.color;
            }
            for i in 0..step_y.abs() {
                sb.base[((y + i * dy.signum()) * sb.w as i64 + x) as usize] = self.color;
            }
            x += step_x;
            y += step_y;
            step_count -= 1;
        }
    }
}

pub struct Image {
    pub file: File,
    pub width: u32,
    pub height: u32,
    pub start: u64,
}

impl Image {
    pub fn new(file: File) -> Result<Image, &'static str> {
        if unsafe { *(file.ptr as *const u16) } != 0x3650 {
            return Err("Invalid format");
        }

        // Skip comment
        let mut start = 2;
        loop {
            if unsafe { *((file.ptr + start) as *const u8) } == '\n' as u8 {
                start += 1;
                if unsafe { *((file.ptr + start) as *const u8) } == '#' as u8 {
                    loop {
                        start += 1;
                        if unsafe { *((file.ptr + start) as *const u8) } == '\n' as u8 {
                            start += 1;
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }

        // Get width
        let mut buffer: [u8; 10] = [0; 10];
        let mut i = 0;
        loop {
            buffer[i] = unsafe { *((file.ptr + start) as *const u8) };
            if buffer[i] < '0' as u8 || buffer[i] > '9' as u8 {
                buffer[i] = 0;
                break;
            }
            i += 1;
            start += 1;
        }
        let mut width: u32 = 0;
        let ten: u32 = 10;
        for j in 0..i {
            width += (buffer[j] as u32 - '0' as u32) * (ten.pow((i - j - 1) as u32));
        }

        // Get height
        let mut buffer: [u8; 10] = [0; 10];
        let mut i = 0;
        start += 1;
        loop {
            buffer[i] = unsafe { *((file.ptr + start) as *const u8) };
            if buffer[i] < '0' as u8 || buffer[i] > '9' as u8 {
                buffer[i] = 0;
                break;
            }
            i += 1;
            start += 1;
        }
        let mut height: u32 = 0;
        let ten: u32 = 10;
        for j in 0..i {
            height += (buffer[j] as u32 - '0' as u32) * (ten.pow((i - j - 1) as u32));
        }

        // Find start of data
        start += 5;

        let out = Image {
            file,
            width,
            height,
            start,
        };
        Ok(out)
    }
}

impl Image {
    pub fn draw(&self, sb: &mut ScreenBuffer, sx: i64, sy: i64, width: u64, height: u64) {
        // Bounds checking
        let left = i64::clamp(sx, 0, sb.w as i64) as u64;
        let top = i64::clamp(sy, 0, sb.h as i64) as u64;
        let right = i64::clamp(sx + width as i64, 0, sb.w as i64) as u64;
        let bottom = i64::clamp(sy + height as i64, 0, sb.h as i64) as u64;

        let mut start_x = if sx < 0 { -sx } else { 0 } as u64;
        let mut start_y = if sy < 0 { -sy } else { 0 } as u64;
        for i in top..bottom {
            let y = (start_y + i - top) as u64 * self.height as u64 / height;
            for j in left..right {
                let x = (start_x + j - left) as u64 * self.width as u64 / width;
                let color = self.get_pixel(x, y);
                if color != 0xff00ff {
                    sb.base[(i as u64 * sb.w + j as u64) as usize] = color;
                }
            }
        }
    }

    fn get_pixel(&self, x: u64, y: u64) -> u32 {
        let r = unsafe {
            *((self.file.ptr + self.start + (y as u64 * self.width as u64 + x as u64) * 3 + 0)
                as *const u8)
        };
        let g = unsafe {
            *((self.file.ptr + self.start + (y as u64 * self.width as u64 + x as u64) * 3 + 1)
                as *const u8)
        };
        let b = unsafe {
            *((self.file.ptr + self.start + (y as u64 * self.width as u64 + x as u64) * 3 + 2)
                as *const u8)
        };
        ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    }
}

pub struct Circle {
    pub x: i64,
    pub y: i64,
    pub r: u64,
    pub color: u32,
}

impl Circle {
    pub fn draw(&self, sb: &mut ScreenBuffer) {
        // Bounds checking
        let rect_x = i64::clamp(self.x - self.r as i64, 0, sb.w as i64);
        let rect_y = i64::clamp(self.y - self.r as i64, 0, sb.h as i64);
        let rect_right = i64::clamp(self.x + self.r as i64, 0, sb.w as i64);
        let rect_bottom = i64::clamp(self.y + self.r as i64, 0, sb.h as i64);

        let r2 = self.r.pow(2);
        for i in rect_y..rect_bottom {
            for j in rect_x..rect_right {
                let distance = (j - self.x).pow(2) + (i - self.y).pow(2);
                if distance < r2 as i64 {
                    unsafe {
                        sb.base[(i * sb.w as i64 + j) as usize] = self.color;
                    }
                }
            }
        }
    }
}

pub struct Rectangle {
    pub rect: Rect,
    pub color: u32,
}

impl Rectangle {
    pub fn draw(&self, sb: &mut ScreenBuffer) {
        // Bounds checking
        if self.rect.x >= sb.w as i64 || self.rect.y >= sb.w as i64 {
            return;
        }
        let x = i64::max(self.rect.x, 0);
        let y = i64::max(self.rect.y, 0);
        let mut right = i64::min(self.rect.x + self.rect.width as i64, sb.w as i64);
        let mut bottom = i64::min(self.rect.y + self.rect.height as i64, sb.h as i64);

        for i in y..bottom {
            for j in x..right {
                unsafe {
                    sb.base[(i * sb.w as i64 + j) as usize] = self.color;
                }
            }
        }
    }
}

pub struct ScreenBuffer<'a> {
    pub x: u64,
    pub y: u64,
    pub w: u64,
    pub h: u64,
    pub base: &'a mut [u32],
}

impl ScreenBuffer<'_> {
    pub fn new<'a>(x: u64, y: u64, w: u64, h: u64, base: &'a mut [u32]) -> ScreenBuffer<'a> {
        ScreenBuffer { x, y, w, h, base }
    }

    pub fn put(&self) {
        unsafe {
            asm!(
                "int 0x41",
                in("rax") self.base.as_ptr(),
                in("rcx") self.x,
                in("rdx") self.y,
                in("r8") self.w,
                in("r9") self.h,
            );
        }
    }

    /*#[inline]
    pub fn bound_checked_put_pixel(&mut self, x: i64, y: i64, color: u32) {
        if x < 0 || x >= self.w || y < 0 || y >= self.h {
            return;
        }

        self.base[(y*self.w as isize + x) as usize] = color;
    }*/

    pub fn clear(&mut self, color: u32) {
        self.base.fill(color);
    }

    pub fn copy_from_screen_buffer(&mut self, sb: &ScreenBuffer) {
        if sb.w != self.w || sb.h != self.h {
            panic!("Could not copy frame buffer");
        }

        unsafe {
            core::ptr::copy_nonoverlapping(
                sb.base as *const [u32] as *const u32,
                self.base as *mut [u32] as *mut u32,
                (self.w * self.h) as usize,
            );
        }
    }
}

pub struct Screen {
    pub width: u64,
    pub height: u64,
}

pub fn get_screen() -> Screen {
    let mut screen = Screen {
        width: 0,
        height: 0,
    };
    unsafe {
        asm!(
            "int 0x42",
            out("rax") screen.width,
            out("rcx") screen.height,
        );
    }
    screen
}
