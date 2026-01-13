use embedded_hal_async::delay::DelayNs;

use crate::{ColorFormat, DisplayBus, DisplayError, SingleColor};

pub mod initseq;
pub mod reset;

/// Display orientation.
pub enum Orientation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[allow(async_fn_in_trait)]
/// Trait for display panels.
pub trait Panel<B: DisplayBus> {
    /// Initializes the panel.
    async fn init<D: DelayNs>(&mut self, bus: &mut B, delay: D) -> Result<(), B::Error>;

    /// Returns the panel resolution (width, height).
    fn size(&self) -> (u16, u16);

    // fn offset(&self) -> (u16, u16);

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

    /// Writes pixels to the specified area.
    /// 
    /// # Arguments
    /// * `bus` - The display bus interface.
    /// * `x` - Start X coordinate.
    /// * `y` - Start Y coordinate.
    /// * `w` - Width of the area.
    /// * `h` - Height of the area.
    /// * `buffer` - Pixel data.
    async fn write_pixels(&mut self, 
        bus: &mut B,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        buffer: &[u8],
    ) -> Result<(), DisplayError<B::Error>>;

    async fn fill_solid(&mut self, 
        bus: &mut B,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        color: SingleColor,
    ) -> Result<(), DisplayError<B::Error>>;

    /// Fills the entire screen with a solid color.
    async fn fill_screen(&mut self, bus: &mut B, color: SingleColor) -> Result<(), DisplayError<B::Error>> {
        let (w, h) = self.size();
        self.fill_solid(bus, 0, 0, w - 1, h - 1, color).await
    }

    /// Verifies the panel ID (if supported).
    async fn verify_id(&mut self, 
        bus: &mut B,
    ) -> Result<bool, DisplayError<B::Error>> {
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


pub const fn address_window_param_u8(start: u16, end: u16, offset: u16) -> [u8; 4] {
    let s = (start + offset).to_be_bytes();
    let e = (end + offset).to_be_bytes();
    [s[0], s[1], e[0], e[1]]
}
