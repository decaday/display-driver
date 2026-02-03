use embedded_hal_async::delay::DelayNs;

use crate::{bus::BusRead, ColorFormat, DisplayBus, DisplayError};

pub mod initseq;
pub mod reset;

/// Display orientation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[allow(async_fn_in_trait)]
/// A trait representing a specific display panel model (e.g., ST7789, ILI9341).
///
/// While [`DisplayBus`] handles *how* data is sent to the screen, this `Panel` trait handles *what* is sent.
/// It encapsulates the specific command set and initialization sequence required by the display controller IC.
pub trait Panel<B: DisplayBus> {
    const CMD_LEN: usize;

    /// The specific command byte(s) used to initiate a pixel write operation to the display's RAM.
    ///
    /// For many MIPI DCS compliant displays, this is `0x2C` (RAMWR). Defining it as a constant allow
    /// the driver to start a pixel transfer efficiently without constructing the command at runtime.
    ///
    /// Note: We can't use `[u8; Self::CMD_LEN]` in stable Rust constants yet, so we use a reference slice
    /// `&PIXEL_WRITE_CMD[0..P::CMD_LEN]` when using this.
    const PIXEL_WRITE_CMD: [u8; 4];

    /// Returns the display width, accounting for orientation.
    fn width(&self) -> u16;

    /// Returns the display height, accounting for orientation.
    fn height(&self) -> u16;

    /// Returns the display size (width, height), accounting for orientation.
    fn size(&self) -> (u16, u16) {
        (self.width(), self.height())
    }

    /// Returns the X coordinate alignment requirements for this panel.
    fn x_alignment(&self) -> u16 {
        1
    }

    /// Returns the Y coordinate alignment requirements for this panel.
    fn y_alignment(&self) -> u16 {
        1
    }

    /// Initializes the panel.
    async fn init<D: DelayNs>(&mut self, bus: &mut B, delay: D) -> Result<(), B::Error>;

    /// Sets the active drawing window on the display.
    ///
    /// This method translates the abstract coordinates (x0, y0, x1, y1) into the specific "Column Address Set"
    /// and "Page Address Set" commands understood by the display controller.
    ///
    /// Note: For some monochrome displays or AMOLED panels, coordinates must be aligned to `self.x_alignment()`
    /// and `self.y_alignment()`.
    async fn set_window(
        &mut self,
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), DisplayError<B::Error>>;

    /// Sets the window to the full screen size.
    async fn set_full_window(&mut self, bus: &mut B) -> Result<(), DisplayError<B::Error>> {
        self.set_window(bus, 0, 0, self.width() - 1, self.height() - 1)
            .await
    }

    /// Check the panel ID (if supported).
    async fn check_id(&mut self, bus: &mut B) -> Result<bool, DisplayError<B::Error>>
    where
        B: BusRead,
    {
        let _ = bus;
        Err(DisplayError::Unsupported)
    }

    /// Sets the display orientation.
    async fn set_orientation(
        &mut self,
        bus: &mut B,
        orientation: Orientation,
    ) -> Result<(), DisplayError<B::Error>> {
        let _ = (bus, orientation);
        Err(DisplayError::Unsupported)
    }

    /// Configures the pixel color format (e.g., RGB565, RGB888).
    ///
    /// This updates the display controller's interface pixel format setting to match the data being sent.
    async fn set_color_format(
        &mut self,
        bus: &mut B,
        color_format: ColorFormat,
    ) -> Result<(), DisplayError<B::Error>>;
}

#[allow(async_fn_in_trait)]
/// An optional trait for setting the panel’s own brightness via commands.
///
/// Note: Using a PWM pin to implement this trait is not recommended.
pub trait PanelSetBrightness<B: DisplayBus>: Panel<B> {
    /// Sets the panel’s own brightness.
    ///
    /// The brightness is represented as a value between 0 and 255, where 0 is the minimum brightness and 255 is the maximum brightness.
    async fn set_brightness(
        &mut self,
        bus: &mut B,
        brightness: u8,
    ) -> Result<(), DisplayError<B::Error>>;
}
