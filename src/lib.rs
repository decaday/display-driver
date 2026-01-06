#![no_std]

pub mod panel;
pub mod display_bus;
#[cfg(feature = "display-interface")]
pub mod di;

use panel::{Orientation, Panel, PanelError};
use display_bus::{DisplayBus, Flags};

pub struct DisplayDriver<B: DisplayBus, P: Panel<B>> {
    pub bus: B,
    pub panel: P,
}

impl<B: DisplayBus, P: Panel<B>> DisplayDriver<B, P> {
    pub fn new(bus: B, panel: P) -> Self {
        Self { bus, panel }
    }

    pub async fn write_pixels(&mut self, 
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        pixels: &[u8]
    ) -> Result<(), B::Error> {
        self.panel.start_write_pixels(&mut self.bus, x0, y0, x1, y1).await?;
        self.bus.write_pixels(x0, y0, x1, y1, pixels).await?;
        self.panel.end_write_pixels(&mut self.bus).await
    }

    pub async fn set_orientation(&mut self, 
        orientation: Orientation,
    ) -> Result<(), PanelError<B::Error>> {
        self.panel.set_orientation(&mut self.bus, orientation).await
    }
}