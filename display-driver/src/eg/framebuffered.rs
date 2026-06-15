use crate::{
    bus::DisplayBus,
    color::ColorFormat,
    panel::{Orientation, Panel, PanelSetBrightness},
    Area, DisplayDriver, DisplayError, FrameControl,
};
use delegate::delegate;
use embedded_graphics::framebuffer::Framebuffer;
use embedded_graphics_core::{
    draw_target::DrawTarget, geometry::OriginDimensions, geometry::Size, pixelcolor::PixelColor,
    Pixel,
};
use embedded_hal_async::delay::DelayNs;

/// A display driver that buffers drawing operations in a framebuffer.
///
/// This struct wraps an underlying [`DisplayDriver`] and a mutable reference to an
/// `embedded-graphics` [`Framebuffer`]. It implements the [`DrawTarget`] trait, forwarding
/// all drawing operations to the framebuffer, and provides methods to flush the buffered
/// pixels to the display.
///
/// It supports both full-screen refreshes and partial updates (when the framebuffer covers
/// only a specific sub-area of the screen).
///
/// # Usage Examples
///
/// ## Full Screen Refreshing
///
/// By default, using the [`new`](Self::new) constructor configures the driver to refresh the full area
/// corresponding to the framebuffer size (`W` x `H`) starting at the origin (0, 0):
///
/// ```ignore
/// // Initialize driver and framebuffer
/// let mut fb_display = FrameBufferedDisplayDriver::new(driver, &mut framebuffer);
///
/// // Draw elements onto the framebuffer
/// fb_display.clear(BinaryColor::Off).unwrap();
/// Circle::new(Point::new(10, 10), 20)
///     .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
///     .draw(&mut fb_display)
///     .unwrap();
///
/// // Flush the changes to the screen
/// fb_display.flush().await.unwrap();
/// ```
///
/// ## Partial Screen Refreshing
///
/// If the framebuffer is only used to update a specific sub-region of the screen, use [`new_partial`](Self::new_partial):
///
/// ```ignore
/// let area = Area::new(50, 50, 100, 100); // W=100, H=100
/// let mut fb_display = FrameBufferedDisplayDriver::new_partial(driver, area, &mut framebuffer)?;
///
/// // Flush to the specified sub-region of the screen
/// fb_display.flush().await.unwrap();
/// ```
pub struct FrameBufferedDisplayDriver<
    'a,
    B,
    P,
    C,
    R,
    BO,
    const W: usize,
    const H: usize,
    const N: usize,
> where
    B: DisplayBus,
    P: Panel<B>,
    C: PixelColor<Raw = R>,
{
    pub driver: DisplayDriver<B, P>,
    area: Area,
    framebuffer: &'a mut Framebuffer<C, R, BO, W, H, N>,
}

impl<'a, B, P, C, R, BO, const W: usize, const H: usize, const N: usize>
    FrameBufferedDisplayDriver<'a, B, P, C, R, BO, W, H, N>
where
    B: DisplayBus,
    P: Panel<B>,
    C: PixelColor<Raw = R>,
{
    /// Creates a new `FrameBufferedDisplayDriver`.
    ///
    /// By default, the display area is set to the full size of the framebuffer (`W` x `H`),
    /// starting at the origin (0, 0).
    ///
    /// # Arguments
    /// * `driver` - The underlying display driver.
    /// * `framebuffer` - A mutable reference to the framebuffer of size `W` x `H`.
    pub fn new(
        driver: DisplayDriver<B, P>,
        framebuffer: &'a mut Framebuffer<C, R, BO, W, H, N>,
    ) -> Self {
        Self {
            driver,
            area: Area::from_origin(W as u16, H as u16),
            framebuffer,
        }
    }

    /// Creates a new `FrameBufferedDisplayDriver` with a specific partial display area.
    ///
    /// This is used when the framebuffer corresponds to a sub-region of the screen.
    ///
    /// # Errors
    /// Returns `DisplayError::InvalidArgs` if the size of the `area` does not exactly match
    /// the dimensions of the framebuffer (`W` x `H`).
    ///
    /// # Arguments
    /// * `driver` - The underlying display driver.
    /// * `area` - The sub-region on the display screen where the framebuffer should be drawn.
    /// * `framebuffer` - A mutable reference to the framebuffer.
    pub fn new_partial(
        driver: DisplayDriver<B, P>,
        area: Area,
        framebuffer: &'a mut Framebuffer<C, R, BO, W, H, N>,
    ) -> Result<Self, DisplayError<B::Error>> {
        if area.w as usize != W || area.h as usize != H {
            return Err(DisplayError::InvalidArgs);
        }

        Ok(Self {
            driver,
            area,
            framebuffer,
        })
    }

    /// Updates the display area where the framebuffer will be drawn.
    ///
    /// # Errors
    /// Returns `DisplayError::InvalidArgs` if the new area does not exactly match
    /// the dimensions of the framebuffer.
    pub fn set_area(&mut self, area: Area) -> Result<(), DisplayError<B::Error>> {
        if area.w as usize != W || area.h as usize != H {
            return Err(DisplayError::InvalidArgs);
        }
        self.area = area;
        Ok(())
    }

    /// Gets the current display area.
    pub fn area(&self) -> Area {
        self.area
    }

    /// Gets a reference to the underlying framebuffer.
    pub fn framebuffer(&self) -> &Framebuffer<C, R, BO, W, H, N> {
        self.framebuffer
    }

    /// Gets a mutable reference to the underlying framebuffer.
    pub fn framebuffer_mut(&mut self) -> &mut Framebuffer<C, R, BO, W, H, N> {
        self.framebuffer
    }

    /// Flushes the underlying framebuffer to the display at the current display area.
    pub async fn flush(&mut self) -> Result<(), DisplayError<B::Error>> {
        self.flush_with_frame_control(FrameControl::new_standalone())
            .await
    }

    /// Flushes the underlying framebuffer to the display at the current display area
    /// using custom frame control settings.
    ///
    /// Frame control settings allow managing options such as whether this is a standalone
    /// write or part of a sequence of writes, which can be useful for double-buffering,
    /// tearing effect (TE) synchronization, or multi-part updates.
    ///
    /// # Arguments
    /// * `frame_control` - The frame synchronization/control flags to use for the write.
    pub async fn flush_with_frame_control(
        &mut self,
        frame_control: FrameControl,
    ) -> Result<(), DisplayError<B::Error>> {
        self.driver
            .write_pixels(self.area, frame_control, self.framebuffer.data())
            .await
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

impl<'a, B, P, C, R, BO, const W: usize, const H: usize, const N: usize>
    FrameBufferedDisplayDriver<'a, B, P, C, R, BO, W, H, N>
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

impl<'a, B, P, C, R, BO, const W: usize, const H: usize, const N: usize> OriginDimensions
    for FrameBufferedDisplayDriver<'a, B, P, C, R, BO, W, H, N>
where
    B: DisplayBus,
    P: Panel<B>,
    C: PixelColor<Raw = R>,
{
    fn size(&self) -> Size {
        self.framebuffer.size()
    }
}

impl<'a, B, P, C, R, BO, const W: usize, const H: usize, const N: usize> DrawTarget
    for FrameBufferedDisplayDriver<'a, B, P, C, R, BO, W, H, N>
where
    B: DisplayBus,
    P: Panel<B>,
    C: PixelColor<Raw = R>,
    Framebuffer<C, R, BO, W, H, N>: DrawTarget<Color = C>,
{
    type Color = C;
    type Error = <Framebuffer<C, R, BO, W, H, N> as DrawTarget>::Error;

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
