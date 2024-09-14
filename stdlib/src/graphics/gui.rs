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
pub struct GuiRect<'a> {
    pub x: Coordinate,
    pub y: Coordinate,
    pub width: Dimension,
    pub height: Dimension,
    pub padding_x: u64,
    pub padding_y: u64,
    pub margin_x: u64,
    pub margin_y: u64,
    pub fill: Fill<'a>,
    pub layout: Layout,
    pub children: Vec<GuiRect<'a>>,
}

impl Default for GuiRect<'_> {
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

impl GuiRect<'_> {
    fn draw(&self, parent: GuiRectResolved, sb: &mut ScreenBuffer) -> GuiRectResolved {
        let mut resolved = self.resolve(parent);

        // Draw self
        match self.fill {
            Fill::Image(img) => {
                img.draw(sb, resolved.x, resolved.y, resolved.width, resolved.height);
            }
            _ => {
                // Bounds checking
                let left = i64::clamp(resolved.x, 0, sb.w as i64) as u64;
                let top = i64::clamp(resolved.y, 0, sb.h as i64) as u64;
                let right = i64::clamp(resolved.x + resolved.width as i64, 0, sb.w as i64) as u64;
                let bottom = i64::clamp(resolved.y + resolved.height as i64, 0, sb.h as i64) as u64;

                for i in top..bottom {
                    for j in left..right {
                        let color = match self.fill {
                            Fill::Solid(color) => color,
                            _ => 0xff00ff,
                        };
                        sb.base[(i * sb.w + j) as usize] = color;
                    }
                }
            }
        }

        // Draw children
        resolved.x += self.padding_x as i64;
        resolved.y += self.padding_y as i64;
        for child in &self.children {
            resolved.x += child.margin_x as i64;
            resolved.y += child.margin_y as i64;
            let child_resolved = child.draw(resolved, sb);
            resolved.x -= child.margin_x as i64;
            resolved.y -= child.margin_y as i64;

            // Advance in layout
            match self.layout {
                Layout::Horizontal => {
                    resolved.x += child_resolved.width as i64 + child.margin_x as i64 * 2;
                }
                Layout::Vertical => {
                    resolved.y += child_resolved.height as i64 + child.margin_y as i64 * 2;
                }
            }
        }

        resolved
    }

    fn resolve(&self, parent: GuiRectResolved) -> GuiRectResolved {
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

        let relative_width = match self.width {
            Dimension::Relative => match self.fill {
                Fill::Image(img) => Some((img.width, img.height)),
                _ => panic!("Relative dimension on non-image"),
            },
            _ => None,
        };
        let relative_height = match self.height {
            Dimension::Relative => match self.fill {
                Fill::Image(img) => Some((img.width, img.height)),
                _ => panic!("Relative dimension on non-image"),
            },
            _ => None,
        };

        if relative_height.is_some() && relative_width.is_some() {
            let img_dimensions = relative_width.unwrap();
            resolved.width = img_dimensions.0 as u64;
            resolved.height = img_dimensions.1 as u64;
        } else if relative_width.is_some() {
            let img_dimensions = relative_width.unwrap();
            resolved.width =
                (img_dimensions.0 as f32 / img_dimensions.1 as f32 * resolved.height as f32) as u64;
        } else if relative_height.is_some() {
            let img_dimensions = relative_height.unwrap();
            resolved.height =
                (img_dimensions.1 as f32 / img_dimensions.0 as f32 * resolved.width as f32) as u64;
        }
        resolved
    }
}

#[derive(Clone, Copy)]
pub enum Dimension {
    Absolute(u64),   // From value
    Relative,        // Only applicable to images. Maintain aspect ratio with the other dimension
    Percentage(f32), // Percentage of parent, from 0.0 to 1.0
}

impl Dimension {
    pub fn resolve(&self, parent: u64, parent_padding: u64, margin: u64) -> u64 {
        match *self {
            Self::Absolute(val) => val,
            Self::Relative => 0,
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
pub enum Fill<'a> {
    Solid(u32),
    Image(&'a Image),
}

#[derive(Clone, Copy)]
pub enum Layout {
    Horizontal,
    Vertical,
}
