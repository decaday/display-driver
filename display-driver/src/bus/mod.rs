#[cfg(feature = "display-interface")]
mod display_interface_impl;

pub mod qspi_flash;
pub use qspi_flash::QspiFlashBus;

pub mod batch_fill;
pub use batch_fill::BatchFillBus;

use crate::{DisplayError, SingleColor};

// /// Configuration for the display bus.
// pub struct Config {
//     /// Screen width in pixels.
//     pub screen_width: u16,
//     /// Screen height in pixels.
//     pub screen_height: u16,

//     /// Color format (e.g., RGB565).
//     pub color_format: ColorFormat,
//     /// Command size in bytes (usually 1).
//     pub cmd_size_bytes: u8,
//     // msb: bool
// }

#[allow(async_fn_in_trait)]
/// A simplified interface for display buses (e.g., SPI, I2C).
pub trait SimpleDisplayBus {
    /// Error type for bus operations.
    type Error;

    // fn configure(&mut self, config: Config) -> Result<(), DisplayError<Self::Error>> {
    //     let _ = config;
    //     Ok(())
    // }

    /// Writes a sequence of commands.
    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    /// Writes data bytes.
    async fn write_data(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Writes a command followed by its parameters.
    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error> {
        self.write_cmds(cmd).await?;
        self.write_data(params).await
    }

    /// Reads data from the display (optional).
    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        let (_, _, _) = (cmd, params, buffer);
        Err(DisplayError::Unsupported)
    }

    /// Sets the hardware reset state (optional).
    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        let _ = reset;
        Err(DisplayError::Unsupported)
    }
}

/// Metadata about the pixel data transfer.
#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    /// Start X coordinate.
    pub x: u16,
    /// Start Y coordinate.
    pub y: u16,
    /// Width of the area being written.
    pub w: u16,
    /// Height of the area being written.
    pub h: u16,
}

#[allow(async_fn_in_trait)]
/// Core trait for display bus implementations.
pub trait DisplayBus {
    /// Error type for bus operations.
    type Error;

    // fn configure(&mut self, config: Config) -> Result<(), DisplayError<Self::Error>>;

    /// Writes a sequence of commands.
    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    /// Writes a command followed by its parameters.
    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error>;

    /// Writes pixel data to the display.
    async fn write_pixels(&mut self, cmd: &[u8], data: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>>;

    /// Fills a region with a solid color.
    ///
    /// This operation can often be hardware accelerated (e.g., 2D DMA with non-incrementing source).
    /// If using `SimpleDisplayBus`, note that the default implementation uses a 16-byte stack buffer.
    /// If larger batches are desired, consider wrapping with `BatchFillBus`.
    async fn fill_solid(&mut self, cmd: &[u8], color: SingleColor, metadata: Metadata) -> Result<(), DisplayError<Self::Error>>;

    /// Reads data from the display (optional).
    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        let (_, _, _) = (cmd, params, buffer);
        Err(DisplayError::Unsupported)
    }

    /// Sets the hardware reset state (optional).
    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        let _ = reset;
        Err(DisplayError::Unsupported)
    }
}

impl<T: SimpleDisplayBus> DisplayBus for T {
    type Error = T::Error;

    // fn configure(&mut self, config: Config) -> Result<(), DisplayError<Self::Error>> {
    //     T::configure(self, config)
    // }

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

    async fn fill_solid(&mut self, cmd: &[u8], color: SingleColor, metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        self.write_cmds(cmd).await.map_err(DisplayError::BusError)?;

        let pixel_size = color.format.size_bytes() as usize;
        let total_pixels = metadata.w as usize * metadata.h as usize;
        let mut remaining_pixels = total_pixels;

        // Use a small fixed-size buffer on the stack to minimize overhead
        let mut buffer = [0u8; 16];
        
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
            self.write_data(&buffer[0..byte_count]).await.map_err(DisplayError::BusError)?;
            remaining_pixels -= current_pixels;
        }

        Ok(())
    }

    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        T::read_data(self, cmd, params, buffer).await
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        T::set_reset(self, reset)
    }
}



