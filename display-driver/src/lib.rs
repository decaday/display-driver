#![no_std]

pub mod area;
pub mod bus;
pub mod color;
pub mod panel;

pub use crate::area::Area;
use crate::bus::BusNonAtomicCmdData;
pub use crate::bus::{BusAutoFill, DisplayBus, FrameControl, Metadata, SimpleDisplayBus};
pub use color::{ColorFormat, ColorType, SolidColor};
pub use panel::{reset::LCDResetOption, Orientation, Panel};

use embedded_hal_async::delay::DelayNs;

#[derive(Debug)]
/// A unified error type identifying what went wrong during a display operation.
pub enum DisplayError<E> {
    /// Error propagated from the underlying bus.
    BusError(E),
    /// The requested operation is not supported by the display or driver.
    Unsupported,
    /// Parameter is out of valid range.
    OutOfRange,
    /// Invalid arguments.
    InvalidArgs,
    /// The area is unaligned.
    UnalignedArea,
}

impl<E> From<E> for DisplayError<E> {
    fn from(error: E) -> Self {
        Self::BusError(error)
    }
}

/// A builder for configuring and initializing a [`DisplayDriver`].
///
/// Use [`DisplayDriver::builder`] to create a builder, then chain configuration methods
/// and call [`init`](DisplayDriverBuilder::init) to complete initialization.
///
/// # Example
/// ```ignore
/// let mut display = DisplayDriver::builder(bus, panel)
///     .with_color_format(ColorFormat::RGB565)
///     .with_orientation(Orientation::Deg270)
///     .init(&mut delay).await.unwrap();
/// ```
pub struct DisplayDriverBuilder<B: DisplayBus, P: Panel<B>> {
    bus: B,
    panel: P,
    color_format: Option<ColorFormat>,
    orientation: Option<Orientation>,
}

impl<B: DisplayBus, P: Panel<B>> DisplayDriverBuilder<B, P> {
    /// Creates a new builder with the given bus and panel.
    fn new(bus: B, panel: P) -> Self {
        Self {
            bus,
            panel,
            color_format: None,
            orientation: None,
        }
    }

    /// Sets the color format to be applied during initialization.
    pub fn with_color_format(mut self, color_format: ColorFormat) -> Self {
        self.color_format = Some(color_format);
        self
    }

    /// Sets the orientation to be applied during initialization.
    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// Initializes the display and returns the configured [`DisplayDriver`].
    ///
    /// This method:
    /// 1. Calls the panel's initialization sequence
    /// 2. Applies the color format if configured
    /// 3. Applies the orientation if configured
    pub async fn init<D: DelayNs>(
        mut self,
        delay: &mut D,
    ) -> Result<DisplayDriver<B, P>, DisplayError<B::Error>> {
        self.panel
            .init(&mut self.bus, delay)
            .await
            .map_err(DisplayError::BusError)?;

        if let Some(color_format) = self.color_format {
            self.panel
                .set_color_format(&mut self.bus, color_format)
                .await?;
        }

        if let Some(orientation) = self.orientation {
            self.panel
                .set_orientation(&mut self.bus, orientation)
                .await?;
        }

        Ok(DisplayDriver {
            bus: self.bus,
            panel: self.panel,
        })
    }
}

/// The high-level driver that orchestrates drawing operations.
///
/// This struct acts as the "glue" between the logical [`Panel`] implementation (which knows the command set)
/// and the [`DisplayBus`] (which handles the physical transport). It exposes user-friendly methods
/// for drawing pixels, filling rectangles, and managing the display state.
pub struct DisplayDriver<B: DisplayBus, P: Panel<B>> {
    /// The underlying bus interface used for communication.
    pub bus: B,
    /// The panel.
    pub panel: P,
}

impl<B: DisplayBus, P: Panel<B>> DisplayDriver<B, P> {
    /// Creates a builder for configuring and initializing a display driver.
    ///
    /// # Example
    /// ```ignore
    /// let mut display = DisplayDriver::builder(bus, panel)
    ///     .with_color_format(ColorFormat::RGB565)
    ///     .with_orientation(Orientation::Deg270)
    ///     .init(&mut delay).await.unwrap();
    /// ```
    pub fn builder(bus: B, panel: P) -> DisplayDriverBuilder<B, P> {
        DisplayDriverBuilder::new(bus, panel)
    }

    /// Creates a new display driver directly (without builder).
    ///
    /// Use [`builder`](Self::builder) for a fluent initialization API.
    pub fn new(bus: B, panel: P) -> Self {
        Self { bus, panel }
    }

    /// Initializes the display.
    pub async fn init(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError<B::Error>> {
        self.panel
            .init(&mut self.bus, delay)
            .await
            .map_err(DisplayError::BusError)
    }

    /// Sets the window.
    ///
    /// Use `write_pixels` or `write_frame` if you just want to draw a buffer.
    /// Use `fill_solid_xxx` if you just want to fill an Area.
    pub async fn set_window(&mut self, area: Area) -> Result<(), DisplayError<B::Error>> {
        if self.panel.x_alignment() > 1 || self.panel.y_alignment() > 1 {
            if area.x % self.panel.x_alignment() != 0
                || area.y % self.panel.y_alignment() != 0
                || area.w % self.panel.x_alignment() != 0
                || area.h % self.panel.y_alignment() != 0
            {
                return Err(DisplayError::UnalignedArea);
            }
        }

        let (x1, y1) = area.bottom_right();
        self.panel
            .set_window(&mut self.bus, area.x, area.y, x1, y1)
            .await
    }

    /// Sets the pixel color format.
    pub async fn set_color_format(
        &mut self,
        color_format: ColorFormat,
    ) -> Result<(), DisplayError<B::Error>> {
        self.panel
            .set_color_format(&mut self.bus, color_format)
            .await
    }

    /// Sets the display orientation.
    pub async fn set_orientation(
        &mut self,
        orientation: Orientation,
    ) -> Result<(), DisplayError<B::Error>> {
        self.panel.set_orientation(&mut self.bus, orientation).await
    }

    /// Writes pixels to the specified area.
    pub async fn write_pixels(
        &mut self,
        area: Area,
        frame_control: FrameControl,
        buffer: &[u8],
    ) -> Result<(), DisplayError<B::Error>> {
        self.set_window(area).await?;
        let cmd = &P::PIXEL_WRITE_CMD[0..P::CMD_LEN];
        let metadata = Metadata {
            area: Some(area),
            frame_control,
        };
        self.bus.write_pixels(cmd, buffer, metadata).await
    }

    /// Writes the entire buffer to the display.
    pub async fn write_frame(&mut self, buffer: &[u8]) -> Result<(), DisplayError<B::Error>> {
        self.write_pixels(
            Area::from_origin_size(self.panel.size()),
            FrameControl::new_single(),
            buffer,
        )
        .await
    }
}

impl<B: DisplayBus + BusAutoFill, P: Panel<B>> DisplayDriver<B, P> {
    /// Fills the area with a solid color using bus auto-fill.
    pub async fn fill_solid_via_bus(
        &mut self,
        area: Area,
        frame_control: FrameControl,
        color: SolidColor,
    ) -> Result<(), DisplayError<B::Error>> {
        self.set_window(area).await?;
        let cmd = &P::PIXEL_WRITE_CMD[0..P::CMD_LEN];
        let metadata = Metadata {
            area: Some(area),
            frame_control,
        };
        self.bus.fill_solid(cmd, color, metadata).await
    }

    /// Fills the entire screen with a solid color using bus auto-fill.
    pub async fn fill_screen_via_bus(
        &mut self,
        color: SolidColor,
    ) -> Result<(), DisplayError<B::Error>> {
        self.fill_solid_via_bus(
            Area::from_origin_size(self.panel.size()),
            FrameControl::new_single(),
            color,
        )
        .await
    }
}

impl<B, P> DisplayDriver<B, P>
where
    B: DisplayBus + BusNonAtomicCmdData,
    P: Panel<B>,
{
    /// Fills the area with a solid color.
    pub async fn fill_solid_batch<const N: usize>(
        &mut self,
        area: Area,
        color: SolidColor,
    ) -> Result<(), DisplayError<B::Error>> {
        self.set_window(area).await?;
        let cmd = &P::PIXEL_WRITE_CMD[0..P::CMD_LEN];

        self.bus
            .write_cmds_non_atomic(cmd)
            .await
            .map_err(DisplayError::BusError)?;

        let pixel_size = color.format.size_bytes() as usize;
        let total_pixels = area.total_pixels();
        let mut remaining_pixels = total_pixels;

        let mut buffer = [0u8; N];

        // Calculate how many full pixels fit in the buffer
        let pixels_per_chunk = buffer.len() / pixel_size;

        // Extract the raw bytes for the color based on its size
        let color_bytes = &color.raw[..pixel_size];

        // Pre-fill the buffer with the color pattern
        for i in 0..pixels_per_chunk {
            buffer[i * pixel_size..(i + 1) * pixel_size].copy_from_slice(color_bytes);
        }

        while remaining_pixels > 0 {
            let current_pixels = remaining_pixels.min(pixels_per_chunk);
            let byte_count = current_pixels * pixel_size;
            self.bus
                .write_data_non_atomic(&buffer[0..byte_count])
                .await
                .map_err(DisplayError::BusError)?;
            remaining_pixels -= current_pixels;
        }

        Ok(())
    }

    /// Fills the entire screen with a solid color.
    pub async fn fill_screen_batch<const N: usize>(
        &mut self,
        color: SolidColor,
    ) -> Result<(), DisplayError<B::Error>> {
        self.fill_solid_batch::<N>(Area::from_origin_size(self.panel.size()), color)
            .await
    }
}
