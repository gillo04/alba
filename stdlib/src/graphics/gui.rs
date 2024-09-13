#![allow(unused)]

use super::*;

pub fn draw_gui_tree(root: &GuiRect, sb: &mut ScreenBuffer) {
    let root_resolved = GuiRectResolved {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        padding_x: 0,
        padding_y: 0,
    };

    root.draw(root_resolved, sb);
}

#[derive(Clone)]
pub struct GuiRect {
    pub x: Coordinate,
    pub y: Coordinate,
    pub width: Dimension,
    pub height: Dimension,
    pub padding_x: u64,
    pub padding_y: u64,
    pub margin_x: u64,
    pub margin_y: u64,
    pub fill: Fill,
    pub layout: Layout,
    pub children: Vec<GuiRect>,
}

impl Default for GuiRect {
    fn default() -> Self {
        Self {
            x: Coordinate::Relative(0),
            y: Coordinate::Relative(0),
            width: Dimension::Absolute(0),
            height: Dimension::Absolute(0),
            padding_x: 0,
            padding_y: 0,
            margin_x: 0,
            margin_y: 0,
            fill: Fill::Solid(0),
            layout: Layout::Vertical,
            children: vec![],
        }
    }
}

#[derive(Clone, Copy)]
struct GuiRectResolved {
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
    pub padding_x: u64,
    pub padding_y: u64,
}

impl GuiRect {
    fn draw(&self, parent: GuiRectResolved, sb: &mut ScreenBuffer) {
        let mut resolved = GuiRectResolved {
            x: self.x.resolve(parent.x),
            y: self.y.resolve(parent.y),
            width: self
                .width
                .resolve(parent.width, parent.padding_x, self.margin_x),
            height: self
                .height
                .resolve(parent.height, parent.padding_y, self.margin_y),
            padding_x: self.padding_x,
            padding_y: self.padding_y,
        };

        // Bounds checking
        let left = i64::clamp(resolved.x, 0, sb.w as i64) as u64;
        let top = i64::clamp(resolved.y, 0, sb.h as i64) as u64;
        let right = i64::clamp(resolved.x + resolved.width as i64, 0, sb.w as i64) as u64;
        let bottom = i64::clamp(resolved.y + resolved.height as i64, 0, sb.h as i64) as u64;

        // Draw self
        for i in top..bottom {
            for j in left..right {
                let color = match self.fill {
                    Fill::Solid(color) => color,
                };
                sb.base[(i * sb.w + j) as usize] = color;
            }
        }

        // Draw children
        resolved.x += self.padding_x as i64;
        resolved.y += self.padding_y as i64;
        for child in &self.children {
            resolved.x += child.margin_x as i64;
            resolved.y += child.margin_y as i64;
            child.draw(resolved, sb);
            resolved.x -= child.margin_x as i64;
            resolved.y -= child.margin_y as i64;

            // Advance in layout
            match self.layout {
                Layout::Horizontal => {
                    resolved.x +=
                        child
                            .width
                            .resolve(resolved.width, resolved.padding_x, child.margin_x)
                            as i64
                            + child.margin_x as i64 * 2
                }
                Layout::Vertical => {
                    resolved.y +=
                        child
                            .height
                            .resolve(resolved.height, resolved.padding_y, child.margin_y)
                            as i64
                            + child.margin_y as i64 * 2
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum Dimension {
    Absolute(u64),   // From value
    Relative,        // Inherits from parent
    Percentage(f32), // Percentage of parent, from 0.0 to 1.0
}

impl Dimension {
    pub fn resolve(&self, parent: u64, parent_padding: u64, margin: u64) -> u64 {
        match *self {
            Self::Absolute(val) => val,
            Self::Relative => parent,
            Self::Percentage(percent) => {
                ((parent - parent_padding * 2) as f32 * percent) as u64 - margin * 2
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum Coordinate {
    Absolute(i64), // From value
    Relative(i64), // Added to parent
}

impl Coordinate {
    pub fn resolve(&self, parent: i64) -> i64 {
        match *self {
            Self::Absolute(val) => val,
            Self::Relative(val) => parent + val,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Fill {
    Solid(u32),
}

#[derive(Clone, Copy)]
pub enum Layout {
    Horizontal,
    Vertical,
}
