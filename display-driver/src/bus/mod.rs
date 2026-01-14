#[cfg(feature = "display-interface")]
mod display_interface_impl;

pub mod qspi_flash;
pub use qspi_flash::QspiFlashBus;

use crate::{Area, DisplayError, SingleColor};

#[allow(async_fn_in_trait)]
/// A simplified interface for display buses (e.g., SPI, I2C).
pub trait SimpleDisplayBus {
    /// Error type for bus operations.
    type Error;

    /// Writes a sequence of commands.
    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    /// Writes data bytes.
    async fn write_data(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Writes a command followed by its parameters.
    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error> {
        self.write_cmds(cmd).await?;
        self.write_data(params).await
    }

    /// Reset the screen via the bus (optional).
    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        let _ = reset;
        Err(DisplayError::Unsupported)
    }
}


#[derive(Debug, Clone, Copy, Default)]
pub struct FrameControl {
    pub first: bool,
    pub last: bool,
}

impl FrameControl {
    pub fn new_single() -> Self {
        Self {
            first: true,
            last: true,
        }
    }

    pub fn new_first() -> Self {
        Self {
            first: true,
            last: false,
        }
    }

    pub fn new_last() -> Self {
        Self {
            first: false,
            last: true,
        }
    }
}

/// Metadata about the pixel data transfer.
#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    pub area: Option<Area>,
    pub frame_control: FrameControl,
}

impl Metadata {
    pub fn new_full_screen(w: u16, h: u16) -> Self {
        Self {
            area: Some(Area::from_origin(w, h)),
            frame_control: FrameControl { first: true, last: true }
        }
    }

    pub fn new_stream_continue() -> Self {
        Self {
            area: None,
            frame_control: FrameControl { first: false, last: false },
        }
    }

    pub fn new_from_parts(area: Option<Area>, frame_control: FrameControl)-> Self {
        Self {
            area,
            frame_control,
        }
    }
}

#[allow(async_fn_in_trait)]
/// Core trait for display bus implementations.
pub trait DisplayBus {
    /// Error type for bus operations.
    type Error;

    /// Writes a sequence of commands.
    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    /// Writes a command followed by its parameters.
    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error>;

    /// Writes pixel data to the display.
    async fn write_pixels(&mut self, cmd: &[u8], data: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>>;

    /// Reset the screen via the bus (optional).
    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        let _ = reset;
        Err(DisplayError::Unsupported)
    }
}

#[allow(async_fn_in_trait)]
pub trait BusAutoFill: DisplayBus {
    /// Fills a region with a solid color.
    ///
    /// This operation can often be hardware accelerated (e.g., Graphics Hardware or DMA with non-incrementing source).
    /// If using `SimpleDisplayBus`, note that the default implementation uses a 16-byte stack buffer.
    /// If larger batches are desired, consider wrapping with `BatchFillBus`.
    async fn fill_solid(&mut self, cmd: &[u8], color: SingleColor, metadata: Metadata) -> Result<(), DisplayError<Self::Error>>;
}

#[allow(async_fn_in_trait)]
pub trait BusRead: DisplayBus {
    /// Reads data from the display (optional).
    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        let (_, _, _) = (cmd, params, buffer);
        Err(DisplayError::Unsupported)
    }
}

impl<T: SimpleDisplayBus> DisplayBus for T {
    type Error = T::Error;

    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        T::write_cmds(self, cmd).await
    }

    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error> {
        T::write_cmd_with_params(self, cmd, params).await
    }

    async fn write_pixels(&mut self, cmd: &[u8], data: &[u8], _metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        self.write_cmds(cmd).await.map_err(DisplayError::BusError)?;
        self.write_data(data).await.map_err(DisplayError::BusError)
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        T::set_reset(self, reset)
    }
}
