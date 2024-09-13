#![allow(unused)]

use super::*;
pub struct Image {
    pub file: File,
    pub width: u32,
    pub height: u32,
    pub x: u64,
    pub y: u64,
    pub start: u64,
}

impl Image {
    pub fn new(file: File, x: u64, y: u64) -> Result<Image, &'static str> {
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
            x: 0,
            y: 0,
            start,
        };
        Ok(out)
    }
}

impl Draw for Image {
    fn draw(&self, sb: &mut ScreenBuffer) {
        for i in 0..self.height {
            for j in 0..self.width {
                let r = unsafe {
                    *((self.file.ptr
                        + self.start
                        + (i as u64 * self.width as u64 + j as u64) * 3
                        + 0) as *const u8)
                };
                let g = unsafe {
                    *((self.file.ptr
                        + self.start
                        + (i as u64 * self.width as u64 + j as u64) * 3
                        + 1) as *const u8)
                };
                let b = unsafe {
                    *((self.file.ptr
                        + self.start
                        + (i as u64 * self.width as u64 + j as u64) * 3
                        + 2) as *const u8)
                };
                sb.base[((self.y + i as u64) * sb.w + self.x + j as u64) as usize] =
                    ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            }
        }
    }
}

/*pub struct Image {
    pub file: File,
    pub width: u64,
    pub height: u64,
    pub x: u64,
    pub y: u64,
}

impl Image {
    pub fn new(file: File, x: u64, y: u64) -> Result<Image, &'static str> {
        // Check signature
        let signature = unsafe { *(file.ptr as *const u64) };
        if signature != 0x0a1a0a0d474e5089 {
            return Err("Invalid PNG signature");
        }

        // Iterate through chunks
        let mut size: Option<(u64, u64)> = None;
        let mut bit_depth = 0;
        let mut color_type = 0;
        let mut interlace = 0;
        let mut pos = 8;
        while pos < file.size {
            unsafe {
                let str = core::str::from_raw_parts_mut((file.ptr + 4 + pos) as *mut u8, 4);
                if str == "IHDR" {
                    size = Some((
                        (*((file.ptr + pos + 8) as *const u32)).swap_bytes() as u64,
                        (*((file.ptr + pos + 8 + 4) as *const u32)).swap_bytes() as u64,
                    ));
                    bit_depth = *((file.ptr + pos + 8 + 8) as *const u8);
                    color_type = *((file.ptr + pos + 8 + 9) as *const u8);
                    interlace = *((file.ptr + pos + 8 + 12) as *const u8);

                    if interlace != 0 {
                        return Err("Interlace is unsuppored");
                    }

                    if color_type != 6 {
                        return Err("Unsupported color type");
                    }
                } else if str == "PLTE" {
                    return Err("Palettes are unsupported");
                }

                pos += (*((file.ptr + pos) as *const u32)).swap_bytes() as u64 + 4 * 3;
            }
        }

        let size = match size {
            Some(size) => size,
            None => return Err("IHDR chunk not present"),
        };

        let img = Image {
            file,
            width: size.0,
            height: size.1,
            x,
            y,
        };
        Ok(img)
    }
}

impl Draw for Image {
    fn draw(&self, sb: &mut ScreenBuffer) {
        let mut pos = 8;
        let mut i = self.x + self.y * sb.w;
        let mut zlib_header: Option<u16> = None;
        let mut prev_color = 0;
        let mut line_counter = 0;
        while pos < self.file.size {
            unsafe {
                let size = (*((self.file.ptr + pos) as *const u32)).swap_bytes() as u64;
                let str = core::str::from_raw_parts_mut((self.file.ptr + 4 + pos) as *mut u8, 4);
                if str == "IDAT" {
                    let dptr = (self.file.ptr + pos + 8 + 2) as u64;
                    // println!("p:{:x}", *(dptr as *const u8));
                    for j in 0..size - 2 {
                        if line_counter == 0 {
                            line_counter = self.width;
                            continue;
                        }
                        let color = *((dptr + j) as *const u32);
                        sb.base[i as usize] = color;
                        prev_color = color;
                        i += 1;
                        line_counter -= 1;
                    }
                    return;
                }

                pos += size + 4 * 3;
            }
        }
    }
}*/

#[derive(Clone, Copy)]
pub enum ColorType {
    Grayscale = 0,
    Truecolor = 2,
    Indexed = 3,
    GrayscaleAlpha = 4,
    TruecolorAlpha = 6,
}

pub struct Circle {
    pub x: i64,
    pub y: i64,
    pub r: u64,
    pub color: u32,
}

impl Draw for Circle {
    fn draw(&self, sb: &mut ScreenBuffer) {
        // Bounds checking
        let rect_x = i64::clamp(self.x - self.r as i64, 0, sb.w as i64);
        let rect_y = i64::clamp(self.y - self.r as i64, 0, sb.h as i64);
        let rect_right = i64::clamp(self.x + self.r as i64, 0, sb.w as i64);
        let rect_bottom = i64::clamp(self.y + self.r as i64, 0, sb.h as i64);

        let r2 = self.r.pow(2);
        for i in rect_x..rect_right {
            for j in rect_y..rect_bottom {
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
    pub x: i64,
    pub y: i64,
    pub w: u64,
    pub h: u64,
    pub fill: Fill,
}

impl Draw for Rectangle {
    fn draw(&self, sb: &mut ScreenBuffer) {
        // Bounds checking
        if self.x >= sb.w as i64 || self.y >= sb.w as i64 {
            return;
        }
        let x = i64::max(self.x, 0);
        let y = i64::max(self.y, 0);
        let mut right = i64::min(self.x + self.w as i64, sb.w as i64);
        let mut bottom = i64::min(self.y + self.h as i64, sb.h as i64);

        for i in y..bottom {
            for j in x..right {
                unsafe {
                    sb.base[(i * sb.w as i64 + j) as usize] =
                        self.fill
                            .resolve((j - self.x) as u64, (i - self.y) as u64, self.w, self.h);
                }
            }
        }
    }
}

pub enum Fill {
    Solid(u32),
    Gradient(u32, u32),
}

impl Fill {
    pub fn resolve(&self, x: u64, y: u64, w: u64, h: u64) -> u32 {
        match *self {
            Fill::Solid(color) => color,
            Fill::Gradient(a, b) => {
                let ra = (a >> 16) & 0xff;
                let ga = (a >> 8) & 0xff;
                let ba = a & 0xff;

                let rb = (b >> 16) & 0xff;
                let gb = (b >> 8) & 0xff;
                let bb = b & 0xff;

                let t = x as f32 / w as f32;
                let rc = (ra as f32 + t * (rb as f32 - ra as f32)) as u32;
                let gc = (ga as f32 + t * (gb as f32 - ga as f32)) as u32;
                let bc = (ba as f32 + t * (bb as f32 - ba as f32)) as u32;
                (rc << 16) | (gc << 8) | bc
            }
        }
    }
}

pub trait Draw {
    fn draw(&self, sb: &mut ScreenBuffer);
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

    pub fn clear(&mut self, color: u32) {
        for i in 0..self.h {
            for j in 0..self.w {
                unsafe {
                    self.base[(i * self.w + j) as usize] = color;
                }
            }
        }
    }

    pub fn draw(&mut self, obj: &impl Draw) {
        obj.draw(self);
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
