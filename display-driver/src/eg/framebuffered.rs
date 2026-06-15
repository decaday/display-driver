use crate::{
    bus::DisplayBus,
    color::ColorFormat,
    panel::{Orientation, Panel, PanelSetBrightness},
    DisplayDriver, DisplayError,
};
use delegate::delegate;
use embedded_graphics::framebuffer::Framebuffer;
use embedded_graphics_core::{
    draw_target::DrawTarget, geometry::OriginDimensions, geometry::Size, pixelcolor::PixelColor,
    Pixel,
};
use embedded_hal_async::delay::DelayNs;

/// A display driver that buffers drawing operations in a framebuffer.
pub struct FrameBufferedDisplayDriver<
    'a,
    B,
    P,
    C,
    R,
    BO,
    const W: usize,
    const H: usize,
    const S: usize,
> where
    B: DisplayBus,
    P: Panel<B>,
    C: PixelColor<Raw = R>,
{
    pub driver: DisplayDriver<B, P>,
    framebuffer: &'a mut Framebuffer<C, R, BO, W, H, S>,
}

impl<'a, B, P, C, R, BO, const W: usize, const H: usize, const S: usize>
    FrameBufferedDisplayDriver<'a, B, P, C, R, BO, W, H, S>
where
    B: DisplayBus,
    P: Panel<B>,
    C: PixelColor<Raw = R>,
{
    /// Creates a new `FrameBufferedDisplayDriver`.
    pub fn new(
        driver: DisplayDriver<B, P>,
        framebuffer: &'a mut Framebuffer<C, R, BO, W, H, S>,
    ) -> Self {
        Self {
            driver,
            framebuffer,
        }
    }

    /// Gets a reference to the underlying framebuffer.
    pub fn framebuffer(&self) -> &Framebuffer<C, R, BO, W, H, S> {
        self.framebuffer
    }

    /// Gets a mutable reference to the underlying framebuffer.
    pub fn framebuffer_mut(&mut self) -> &mut Framebuffer<C, R, BO, W, H, S> {
        self.framebuffer
    }

    /// Flushes the underlying framebuffer to the display.
    pub async fn flush(&mut self) -> Result<(), DisplayError<B::Error>> {
        self.driver.write_frame(self.framebuffer.data()).await
    }

    /// Returns the inner DisplayDriver.
    pub fn into_inner(self) -> DisplayDriver<B, P> {
        self.driver
    }

    delegate! {
        to self.driver {
            /// Initializes the display.
            pub async fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError<B::Error>>;
            /// Sets the pixel color format.
            pub async fn set_color_format(&mut self, color_format: ColorFormat) -> Result<(), DisplayError<B::Error>>;
            /// Sets the display orientation.
            pub async fn set_orientation(&mut self, orientation: Orientation) -> Result<(), DisplayError<B::Error>>;
        }
    }
}

impl<'a, B, P, C, R, BO, const W: usize, const H: usize, const S: usize>
    FrameBufferedDisplayDriver<'a, B, P, C, R, BO, W, H, S>
where
    B: DisplayBus,
    P: Panel<B> + PanelSetBrightness<B>,
    C: PixelColor<Raw = R>,
{
    delegate! {
        to self.driver {
            /// Sets the display brightness (if supported by the panel).
            pub async fn set_brightness(&mut self, brightness: u8) -> Result<(), DisplayError<B::Error>>;
        }
    }
}

impl<'a, B, P, C, R, BO, const W: usize, const H: usize, const S: usize> OriginDimensions
    for FrameBufferedDisplayDriver<'a, B, P, C, R, BO, W, H, S>
where
    B: DisplayBus,
    P: Panel<B>,
    C: PixelColor<Raw = R>,
{
    fn size(&self) -> Size {
        self.framebuffer.size()
    }
}

impl<'a, B, P, C, R, BO, const W: usize, const H: usize, const S: usize> DrawTarget
    for FrameBufferedDisplayDriver<'a, B, P, C, R, BO, W, H, S>
where
    B: DisplayBus,
    P: Panel<B>,
    C: PixelColor<Raw = R>,
    Framebuffer<C, R, BO, W, H, S>: DrawTarget<Color = C>,
{
    type Color = C;
    type Error = <Framebuffer<C, R, BO, W, H, S> as DrawTarget>::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.framebuffer.draw_iter(pixels)
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &embedded_graphics_core::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.framebuffer.fill_contiguous(area, colors)
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics_core::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        self.framebuffer.fill_solid(area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.framebuffer.clear(color)
    }
}
