use embedded_graphics::{
    geometry::Point,
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::{PixelColor, Rgb565},
    prelude::*,
    primitives::{Line, Primitive, PrimitiveStyle, Triangle},
    text::Text,
};

/// L-shaped corner markers for display offset verification
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

/// Draw a rotation demo scene with orientation indicators
///
/// This function draws:
/// - L-shaped corner markers (red) to verify display offsets
/// - An upward-pointing arrow (green) indicating the visual top of the buffer
/// - Rotation text showing the current orientation
/// - A triangle marker in the top-right corner (red)
pub fn draw_rotation_scene<D>(target: &mut D, w: usize, h: usize, rot_str: &str)
where
    D: DrawTarget<Color = Rgb565> + Dimensions,
    D::Error: core::fmt::Debug,
{
    target.clear(Rgb565::BLACK).unwrap();

    let text_style = MonoTextStyle::new(&FONT_8X13, Rgb565::WHITE);
    let cx = (w / 2) as i32;
    let cy = (h / 2) as i32;

    // Draw L-shaped markers at the corners to verify offsets
    LShapedMarkers::new(w as i32, h as i32, 5, Rgb565::RED)
        .draw(target)
        .unwrap();

    // Draw Arrow pointing UP (Visual top of the current buffer)
    // Shaft
    Line::new(Point::new(cx, cy + 20), Point::new(cx, cy - 20))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1))
        .draw(target)
        .ok();

    // Head
    Triangle::new(
        Point::new(cx, cy - 20),
        Point::new(cx - 5, cy - 10),
        Point::new(cx + 5, cy - 10),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
    .draw(target)
    .ok();

    // Centered text
    Text::new(rot_str, Point::new(10, 20), text_style)
        .draw(target)
        .ok();

    // Triangle marker in top-right
    Triangle::new(
        Point::new((w as i32) - 10, 10),
        Point::new((w as i32) - 20, 10),
        Point::new((w as i32) - 10, 20),
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 1))
    .draw(target)
    .ok();
}
