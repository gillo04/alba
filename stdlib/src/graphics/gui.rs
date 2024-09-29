#![allow(unused)]

use super::*;

pub fn draw_gui_tree(root: &GuiRect, sb: &mut ScreenBuffer, io: &IoState) {
    let root_resolved = GuiRectResolved {
        rect: Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        },
        padding_x: 0,
        padding_y: 0,
    };

    root.draw(&root_resolved, sb, io);
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
    pub fill_hover: Option<Fill<'a>>,
    pub fill_active: Option<Fill<'a>>,

    pub layout: Layout,
    pub children: Vec<GuiRect<'a>>,
    pub text: Option<(String, &'a Font)>,

    pub on_click: fn(),
}

fn nop() {}

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
            fill_hover: None,
            fill_active: None,

            layout: Layout::Vertical,
            children: vec![],
            text: None,

            on_click: || {},
        }
    }
}

#[derive(Clone, Copy)]
struct GuiRectResolved {
    pub rect: Rect,
    pub padding_x: u64,
    pub padding_y: u64,
}

impl GuiRect<'_> {
    fn draw(
        &self,
        parent: &GuiRectResolved,
        sb: &mut ScreenBuffer,
        io: &IoState,
    ) -> GuiRectResolved {
        let mut resolved = self.resolve(parent);

        // Draw self
        let fill = {
            if resolved
                .rect
                .point_intersection(io.mouse_pos.0 as i64, io.mouse_pos.1 as i64)
            {
                if io.left_button {
                    (self.on_click)();
                    if let Some(f) = self.fill_active {
                        f
                    } else {
                        self.fill
                    }
                } else {
                    if let Some(f) = self.fill_hover {
                        f
                    } else {
                        self.fill
                    }
                }
            } else {
                self.fill
            }
        };

        match fill {
            Fill::Image(img) => {
                img.draw(
                    sb,
                    resolved.rect.x,
                    resolved.rect.y,
                    resolved.rect.width,
                    resolved.rect.height,
                );
            }
            _ => {
                // Bounds checking
                let left = i64::clamp(resolved.rect.x, 0, sb.w as i64) as u64;
                let top = i64::clamp(resolved.rect.y, 0, sb.h as i64) as u64;
                let right =
                    i64::clamp(resolved.rect.x + resolved.rect.width as i64, 0, sb.w as i64) as u64;
                let bottom = i64::clamp(
                    resolved.rect.y + resolved.rect.height as i64,
                    0,
                    sb.h as i64,
                ) as u64;

                for i in top..bottom {
                    for j in left..right {
                        let color = match fill {
                            Fill::Solid(color) => color,
                            _ => 0xff00ff,
                        };
                        sb.base[(i * sb.w + j) as usize] = color;
                    }
                }
            }
        }

        // Draw text
        if self.text.is_some() {
            let text = self.text.as_ref().unwrap();
            let char_bounds = text.1.get_char_bounds();

            let mut lines = vec![];
            let mut line_width = 0;
            let mut previous_split = 0;
            for (i, c) in text.0.chars().enumerate() {
                if c == '\n' {
                    line_width = 0;
                } else {
                    line_width += char_bounds.0;
                    if line_width + char_bounds.0 >= resolved.rect.width {
                        lines.push(&text.0[previous_split..i]);
                        previous_split = i;
                        line_width = 0;
                    }
                }
            }
            if line_width > resolved.rect.width {
                lines.push(&text.0[previous_split..]);
            }

            let final_str = lines.join("\n");
            text.1
                .draw_string(&final_str, resolved.rect.x, resolved.rect.y, 1, 0, sb);
        }

        // Draw children
        resolved.rect.x += self.padding_x as i64;
        resolved.rect.y += self.padding_y as i64;
        for child in &self.children {
            resolved.rect.x += child.margin_x as i64;
            resolved.rect.y += child.margin_y as i64;
            let child_resolved = child.draw(&resolved, sb, io);
            resolved.rect.x -= child.margin_x as i64;
            resolved.rect.y -= child.margin_y as i64;

            // Advance in layout
            match self.layout {
                Layout::Horizontal => {
                    resolved.rect.x += child_resolved.rect.width as i64 + child.margin_x as i64 * 2;
                }
                Layout::Vertical => {
                    resolved.rect.y +=
                        child_resolved.rect.height as i64 + child.margin_y as i64 * 2;
                }
            }
        }

        resolved
    }

    fn resolve(&self, parent: &GuiRectResolved) -> GuiRectResolved {
        let mut resolved = GuiRectResolved {
            rect: Rect {
                x: self.x.resolve(parent.rect.x),
                y: self.y.resolve(parent.rect.y),
                width: self
                    .width
                    .resolve(parent.rect.width, parent.padding_x, self.margin_x),
                height: self
                    .height
                    .resolve(parent.rect.height, parent.padding_y, self.margin_y),
            },
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
            resolved.rect.width = img_dimensions.0 as u64;
            resolved.rect.height = img_dimensions.1 as u64;
        } else if relative_width.is_some() {
            let img_dimensions = relative_width.unwrap();
            resolved.rect.width = (img_dimensions.0 as f32 / img_dimensions.1 as f32
                * resolved.rect.height as f32) as u64;
        } else if relative_height.is_some() {
            let img_dimensions = relative_height.unwrap();
            resolved.rect.height = (img_dimensions.1 as f32 / img_dimensions.0 as f32
                * resolved.rect.width as f32) as u64;
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

#[derive(Clone, Copy)]
pub struct IoState {
    pub mouse_pos: (u64, u64),
    pub left_button: bool,
    pub right_button: bool,
}
