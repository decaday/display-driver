#![no_std]

pub mod panel;
pub mod display_bus;
#[cfg(feature = "display-interface")]
pub mod di;

use display_bus::{DisplayBus};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    Binary,
    Gray2,
    Gray4,
    Gray8,
    RGB565,
    RGB666,
    RGB888,
}

impl ColorFormat {
    pub fn size_bits(self) -> u8 {
        match self {
            ColorFormat::Binary => 1,
            ColorFormat::Gray2 => 2,
            ColorFormat::Gray4 => 4,
            ColorFormat::Gray8 => 8,
            ColorFormat::RGB565 => 16,
            ColorFormat::RGB666 => 18,
            ColorFormat::RGB888 => 24,
        }
    }
}

#[derive(Debug)]
pub enum DisplayError<E> {
    BusError(E),
    Unsupported,
    OutOfRange,
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
