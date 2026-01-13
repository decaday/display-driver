#![no_std]

pub mod panel;
pub mod display_bus;
pub mod color;
#[cfg(feature = "display-interface")]
pub mod di;

pub use display_bus::DisplayBus;
pub use panel::Panel;

pub use color::{ColorFormat, ColorType, SingleColor};


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

// pub struct DisplayDriver<B: DisplayBus, P: Panel<B>> {
//     pub bus: B,
//     pub panel: P,
// }

// impl<B: DisplayBus, P: Panel<B>> DisplayDriver<B, P> {
//     pub fn new(bus: B, panel: P) -> Self {
//         Self { bus, panel }
//     }

//     pub async fn write_pixels(&mut self, 
//         x0: u16,
//         y0: u16,
//         x1: u16,
//         y1: u16,
//         pixels: &[u8]
//     ) -> Result<(), B::Error> {
//         self.panel.write_pixels(&mut self.bus, x0, y0, x1, y1, pixels).await
//     }

//     pub async fn set_orientation(&mut self, 
//         orientation: Orientation,
//     ) -> Result<(), DisplayError<B::Error>> {
//         self.panel.set_orientation(&mut self.bus, orientation).await
//     }
// }
