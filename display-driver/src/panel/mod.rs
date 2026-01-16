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
/// Trait for display panels.
pub trait Panel<B: DisplayBus> {
    const CMD_LEN: usize;
    const X_ALIGNMENT: u16;
    const Y_ALIGNMENT: u16;

    /// Returns the panel resolution (width, height).
    fn size(&self) -> (u16, u16);

    // fn offset(&self) -> (u16, u16);

    /// Note: We can't use [u8; Self::CMD_LEN] in stable
    /// use &pixel_write_command()[0..P::CMD_LEN] instead
    fn pixel_write_command(&mut self) -> [u8; 4];

    /// Initializes the panel.
    async fn init<D: DelayNs>(&mut self, bus: &mut B, delay: D) -> Result<(), B::Error>;

    /// Sets the active window for pixel writing.
    async fn set_window(&mut self, 
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), DisplayError<B::Error>>;

    /// Sets the window to the full screen size.
    async fn set_full_window(&mut self, bus: &mut B) -> Result<(), DisplayError<B::Error>> {
        let (w, h) = self.size();
        self.set_window(bus, 0, 0, w - 1, h - 1).await
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

    /// Sets the color format.
    async fn set_color_format(&mut self, 
        bus: &mut B,
        color_format: ColorFormat,
    ) -> Result<(), DisplayError<B::Error>>;

    /// Sets the brightness (0-255).
    async fn set_brightness(&mut self, 
        bus: &mut B,
        brightness: u8,
    ) -> Result<(), DisplayError<B::Error>> {
        let _ = (bus, brightness);
        Err(DisplayError::Unsupported)
    }

    // async fn set_rgb_order(&mut self, 
    //     bus: &mut B,
    //     rgb_order: bool,
    // ) -> Result<(), DisplayError<B::Error>> {
    //     let _ = (bus, rgb_order);
    //     Err(DisplayError::Unsupported)
    // }
}
