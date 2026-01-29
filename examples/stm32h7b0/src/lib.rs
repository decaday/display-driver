#![no_std]

use embedded_graphics::{
    geometry::Point,
    pixelcolor::PixelColor,
    prelude::*,
    primitives::{Line, Primitive, PrimitiveStyle},
};

pub struct LShapedMarkers<C> {
    pub width: i32,
    pub height: i32,
    pub length: i32,
    pub color: C,
}

impl<C: PixelColor> LShapedMarkers<C> {
    pub fn new(width: i32, height: i32, length: i32, color: C) -> Self {
        Self {
            width,
            height,
            length,
            color,
        }
    }
}

impl<C: PixelColor> Drawable for LShapedMarkers<C> {
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        let style = PrimitiveStyle::with_stroke(self.color, 1);
        let w = self.width;
        let h = self.height;
        let l = self.length;

        // Top-left
        Line::new(Point::new(0, 0), Point::new(l, 0))
            .into_styled(style)
            .draw(target)?;
        Line::new(Point::new(0, 0), Point::new(0, l))
            .into_styled(style)
            .draw(target)?;

        // Top-right
        Line::new(Point::new(w - 1, 0), Point::new(w - 1 - l, 0))
            .into_styled(style)
            .draw(target)?;
        Line::new(Point::new(w - 1, 0), Point::new(w - 1, l))
            .into_styled(style)
            .draw(target)?;

        // Bottom-left
        Line::new(Point::new(0, h - 1), Point::new(l, h - 1))
            .into_styled(style)
            .draw(target)?;
        Line::new(Point::new(0, h - 1), Point::new(0, h - 1 - l))
            .into_styled(style)
            .draw(target)?;

        // Bottom-right
        Line::new(Point::new(w - 1, h - 1), Point::new(w - 1 - l, h - 1))
            .into_styled(style)
            .draw(target)?;
        Line::new(Point::new(w - 1, h - 1), Point::new(w - 1, h - 1 - l))
            .into_styled(style)
            .draw(target)?;

        Ok(())
    }
}
