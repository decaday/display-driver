#![no_std]

pub mod area;
pub mod bus;
pub mod color;
pub mod panel;

pub use crate::area::Area;
pub use crate::bus::{BusAutoFill, DisplayBus, FrameControl, Metadata, SimpleDisplayBus};
pub use color::{ColorFormat, ColorType, SingleColor};
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
    ///
    InvalidArgs,
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
    /// Creates a new display driver.
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
    ///
    /// # Arguments
    /// * `bus` - The display bus interface.
    /// * `buffer` - Pixel data.
    pub async fn write_pixels(
        &mut self,
        area: Area,
        frame_control: FrameControl,
        buffer: &[u8],
    ) -> Result<(), DisplayError<B::Error>> {
        let (x1, y1) = area.bottom_right();
        self.panel
            .set_window(&mut self.bus, area.x, area.y, x1, y1)
            .await?;
        let cmd = &P::PIXEL_WRITE_CMD[0..P::CMD_LEN];
        let metadata = Metadata {
            area: Some(area),
            frame_control,
        };
        self.bus.write_pixels(cmd, buffer, metadata).await
    }

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
    pub async fn fill_solid_via_bus(
        &mut self,
        area: Area,
        frame_control: FrameControl,
        color: SingleColor,
    ) -> Result<(), DisplayError<B::Error>> {
        let (x1, y1) = area.bottom_right();
        self.panel
            .set_window(&mut self.bus, area.x, area.y, x1, y1)
            .await?;
        let cmd = &P::PIXEL_WRITE_CMD[0..P::CMD_LEN];
        let metadata = Metadata {
            area: Some(area),
            frame_control,
        };
        self.bus.fill_solid(cmd, color, metadata).await
    }

    /// Fills the entire screen with a solid color.
    pub async fn fill_screen_via_bus(
        &mut self,
        color: SingleColor,
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
    B: DisplayBus + SimpleDisplayBus,
    P: Panel<B>,
{
    pub async fn fill_solid_batch<const N: usize>(
        &mut self,
        area: Area,
        color: SingleColor,
    ) -> Result<(), DisplayError<B::Error>> {
        let (x1, y1) = area.bottom_right();
        self.panel
            .set_window(&mut self.bus, area.x, area.y, x1, y1)
            .await?;
        let cmd = &P::PIXEL_WRITE_CMD[0..P::CMD_LEN];

        <B as SimpleDisplayBus>::write_cmds(&mut self.bus, cmd)
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
                .write_data(&buffer[0..byte_count])
                .await
                .map_err(DisplayError::BusError)?;
            remaining_pixels -= current_pixels;
        }

        Ok(())
    }

    /// Fills the entire screen with a solid color.
    pub async fn fill_screen_batch<const N: usize>(
        &mut self,
        color: SingleColor,
    ) -> Result<(), DisplayError<B::Error>> {
        self.fill_solid_batch::<N>(Area::from_origin_size(self.panel.size()), color)
            .await
    }
}
