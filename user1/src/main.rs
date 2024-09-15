#![no_std]
#![no_main]

use core::arch::asm;
use core::ops::*;
use stdlib::alloc::vec::*;
use stdlib::alloc::*;
use stdlib::graphics::*;
use stdlib::*;

struct Pt3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Pt3 {
    fn new(x: f64, y: f64, z: f64) -> Pt3 {
        Pt3 { x, y, z }
    }

    fn rotate_x(&mut self, angle: f64) {
        // self.y = self.y * cos(angle) - self.z * sin(angle);
        // self.z = self.y * f64::sin(angle) + self.z * f64::cos(angle);
    }
}

impl Add<Pt3> for Pt3 {
    type Output = Pt3;

    fn add(self, rhs: Pt3) -> Self::Output {
        Pt3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

/*fn sin(angle: f64) -> f64 {
    let sin: f64;
    unsafe {
        asm!(
            "fld qword ptr[{}]",
            "fsin",
            "fstp qword ptr[{}]",
            in(reg) angle,
            out(reg) sin,
        );
    }
    sin
}

fn cos(angle: f64) -> f64 {
    let cos: f64;
    unsafe {
        asm!(
            "fld qword ptr[{}]",
            "fcos",
            "fstp qword ptr [{}]",
            in(reg) angle,
            out(reg) cos,
        );
    }
    cos
}
*/

#[export_name = "_start"]
#[no_mangle]
extern "C" fn main() {
    stdlib::heap::init().unwrap();

    let mut buffer = vec![0x0u32; 500 * 500];
    let mut sbuffer = ScreenBuffer::new(0, 0, 500, 500, &mut buffer[..]);

    // Define cube points
    let mut points = vec![
        Pt3::new(-30.0, -30.0, -30.0),
        Pt3::new(30.0, -30.0, -30.0),
        Pt3::new(30.0, 30.0, -30.0),
        Pt3::new(-30.0, 30.0, -30.0),
        Pt3::new(-30.0, -30.0, 30.0),
        Pt3::new(30.0, -30.0, 30.0),
        Pt3::new(30.0, 30.0, 30.0),
        Pt3::new(-30.0, 30.0, 30.0),
    ];

    let edges = vec![
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
    ];

    let mut prev_time = get_milliseconds_since_startup();
    loop {
        // Draw
        sbuffer.clear(0);
        let mut line = Line {
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
            color: 0xffffff,
        };
        for edge in &edges {
            line.x1 = points[edge.0].x as i64 + sbuffer.w as i64 / 2;
            line.y1 = points[edge.0].y as i64 + sbuffer.h as i64 / 2;
            line.x2 = points[edge.1].x as i64 + sbuffer.w as i64 / 2;
            line.y2 = points[edge.1].y as i64 + sbuffer.h as i64 / 2;
            line.draw(&mut sbuffer);
        }
        sbuffer.put();
        while get_milliseconds_since_startup() - prev_time < 6 {}
        prev_time = get_milliseconds_since_startup();
    }
}
