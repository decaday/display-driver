#![no_std]

pub mod panel;
pub mod bus;
pub mod color;
pub mod area;

pub use panel::Panel;
pub use color::{ColorFormat, ColorType, SingleColor};
pub use crate::area::Area;
pub use crate::bus::{BusAutoFill, FrameControl, Metadata, DisplayBus, SimpleDisplayBus};

#[derive(Debug)]
/// Common errors that can occur during display operations.
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

pub struct DisplayDriver<B: DisplayBus, P: Panel<B>> {
    pub bus: B,
    pub panel: P,
}

impl<B: DisplayBus, P: Panel<B>> DisplayDriver<B, P> {
    /// Init your bus and panel before create a DisplayDriver
    pub fn new(bus: B, panel: P) -> Self {
        Self { bus, panel }
    }

    /// Writes pixels to the specified area.
    /// 
    /// # Arguments
    /// * `bus` - The display bus interface.
    /// * `buffer` - Pixel data.
    pub async fn write_pixels(&mut self,
        area: Area,
        frame_control: FrameControl,
        buffer: &[u8],
    ) -> Result<(), DisplayError<<B as DisplayBus>::Error>> {
        self.panel.set_window(&mut self.bus, area.x, area.y, area.x + area.w - 1, area.y + area.h - 1).await?;
        let cmd = &self.panel.cmd_write_pixels()[0..P::CMD_LEN];
        let metadata = Metadata {area: Some(area), frame_control};
        self.bus.write_pixels(cmd, buffer, metadata).await
    }

    pub async fn write_frame(&mut self, 
        buffer: &[u8],
    ) -> Result<(), DisplayError<B::Error>> {
        self.panel.set_full_window(&mut self.bus).await?;
        let (w, h) = self.panel.size();
        self.write_pixels(Area::new_at_zero(w, h), FrameControl::new_single(), buffer).await
    }
}

impl<B: DisplayBus + BusAutoFill, P: Panel<B>> DisplayDriver<B, P> {
    pub async fn fill_solid_via_bus(&mut self, 
        area: Area,
        frame_control: FrameControl,
        color: SingleColor,
    ) -> Result<(), DisplayError<B::Error>> {
        self.panel.set_window(&mut self.bus, area.x, area.y, area.x + area.w - 1, area.y + area.h - 1).await?;
        let cmd = &self.panel.cmd_write_pixels()[0..P::CMD_LEN];
        let metadata = Metadata {area: Some(area), frame_control};
        self.bus.fill_solid(cmd, color, metadata).await
    }

    /// Fills the entire screen with a solid color.
    pub async fn fill_screen_via_bus(&mut self, color: SingleColor) -> Result<(), DisplayError<B::Error>> {
        let (w, h) = self.panel.size();
        
        self.fill_solid_via_bus(Area::new_at_zero(w, h), FrameControl::new_single(), color).await
    }
}