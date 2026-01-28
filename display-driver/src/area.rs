#[derive(Debug, Clone, Copy)]
pub struct Area {
    /// Start X coordinate.
    pub x: u16,
    /// Start Y coordinate.
    pub y: u16,
    /// Width of the area being written.
    pub w: u16,
    /// Height of the area being written.
    pub h: u16,
}

impl Area {
    pub fn from_origin(w: u16, h: u16) -> Self {
        Self { x: 0, y: 0, w, h }
    }

    pub fn position(&self) -> (u16, u16) {
        (self.x, self.y)
    }

    pub fn size(&self) -> usize {
        self.w as usize * self.h as usize
    }

    pub fn bottom_right(&self) -> (u16, u16) {
        (self.x + self.w - 1, self.y + self.h - 1)
    }
}

// #[derive(Clone, Copy, Debug)]
// pub enum AreaOrSize {
//     Area(Area),
//     // WidthAndHeight(u16, u16),
//     Size(usize)
// }

// impl AreaOrSize {
//     pub fn size(&self) -> usize {
//         match *self {
//             AreaOrSize::Area(area) => area.size(),
//             // AreaOrSize::WidthAndHeight(w, h) => w as usize * h as usize,
//             AreaOrSize::Size(size) => size,
//         }
//     }
// }

#[cfg(feature = "embedded-graphics")]
mod eg_impls {
    use super::*;
    use embedded_graphics_core::primitives::Rectangle;

    impl From<Rectangle> for Area {
        fn from(value: Rectangle) -> Self {
            Area {
                x: value.top_left.x as _,
                y: value.top_left.y as _,
                w: value.size.width as _,
                h: value.size.height as _,
            }
        }
    }
}
