use embedded_hal_async::delay::DelayNs;

use crate::{ColorFormat, DisplayBus, DisplayError, bus::BusRead};

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
    
    const WIDTH: u16;
    const HEIGHT: u16;

    /// Alignment requirements for X and Y coordinates.
    const X_ALIGNMENT: u16;
    const Y_ALIGNMENT: u16;

    /// Initializes the panel.
    async fn init<D: DelayNs>(&mut self, bus: &mut B, delay: D) -> Result<(), B::Error>;

    /// Sets the active drawing window on the display.
    ///
    /// This method translates the abstract coordinates (x0, y0, x1, y1) into the specific "Column Address Set"
    /// and "Page Address Set" commands understood by the display controller.
    /// 
    /// Note: For some monochrome displays or AMOLED panels, coordinates must be aligned to `Self::X_ALIGNMENT` 
    /// and `Self::Y_ALIGNMENT`.
    async fn set_window(&mut self, 
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), DisplayError<B::Error>>;

    /// Sets the window to the full screen size.
    async fn set_full_window(&mut self, bus: &mut B) -> Result<(), DisplayError<B::Error>> {
        self.set_window(bus, 0, 0, Self::WIDTH - 1, Self::HEIGHT - 1).await
    }

    /// Check the panel ID (if supported).
    async fn check_id(&mut self, 
        bus: &mut B,
    ) -> Result<bool, DisplayError<B::Error>> where 
        B: BusRead
    {
        let _ = bus;
        Err(DisplayError::Unsupported)
    }

    /// Sets the display orientation.
    async fn set_orientation(&mut self, 
        bus: &mut B,
        orientation: Orientation,
    ) -> Result<(), DisplayError<B::Error>> {
        let _ = (bus, orientation);
        Err(DisplayError::Unsupported)
    }

    /// Configures the pixel color format (e.g., RGB565, RGB888).
    ///
    /// This updates the display controller's interface pixel format setting to match the data being sent.
    async fn set_color_format(&mut self, 
        bus: &mut B,
        color_format: ColorFormat,
    ) -> Result<(), DisplayError<B::Error>>;
}
