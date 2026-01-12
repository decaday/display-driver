use crate::DisplayError;

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
    async fn fill_solid(&mut self, cmd: &[u8], color: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>>;

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

    async fn fill_solid(&mut self, cmd: &[u8], color: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        self.write_cmds(cmd).await.map_err(DisplayError::BusError)?;

        let pixel_size = color.len();
        let total_pixels = metadata.w as usize * metadata.h as usize;
        let mut remaining_pixels = total_pixels;

        // Use a small fixed-size buffer on the stack to minimize overhead
        let mut buffer = [0u8; 16];
        
        // Calculate how many full pixels fit in the buffer
        let pixels_per_chunk = buffer.len() / pixel_size;
        
        // Pre-fill the buffer with the color pattern
        for i in 0..pixels_per_chunk {
            buffer[i * pixel_size..(i + 1) * pixel_size].copy_from_slice(color);
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

/// Adapter for QSPI buses that require command formatting for flash-like interfaces.
pub struct QspiFlashBus<B: DisplayBus> {
    inner: B,
}

impl<B: DisplayBus> QspiFlashBus<B> {
    /// Creates a new QspiFlashBus wrapper.
    pub fn new(inner: B) -> Self {
        Self { inner }
    }

    /// Formats the command and address for QSPI transfer.
    pub fn to_cmd_and_addr(&self, cmd: &[u8], pixel_data: bool) -> [u8; 4] {
        if cmd.len() != 1 {
            panic!("QSPI command must be 1 byte")
        }

        let flash_command: u8 = if pixel_data {
            0x32
        } else {
            0x02
        };

        [flash_command, 0x00, cmd[0], 0x00]
    }
}

impl<B: DisplayBus> DisplayBus for QspiFlashBus<B> {
    type Error = B::Error;

    // fn configure(&mut self, config: Config) -> Result<(), DisplayError<Self::Error>> {
    //     let mut config = config;
    //     config.cmd_size_bytes = 4;
    //     self.inner.configure(config)
    // }

    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        let cmd = self.to_cmd_and_addr(cmd, false);
        self.inner.write_cmds(&cmd).await
    }

    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error> {
        let cmd = self.to_cmd_and_addr(cmd, false);
        self.inner.write_cmd_with_params(&cmd, params).await
    }

    async fn write_pixels(&mut self, cmd: &[u8], data: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        let cmd = self.to_cmd_and_addr(cmd, true);
        self.inner.write_pixels(&cmd, data, metadata).await
    }

    async fn fill_solid(&mut self, cmd: &[u8], color: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        let cmd = self.to_cmd_and_addr(cmd, true);
        self.inner.fill_solid(&cmd, color, metadata).await
    }

    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        let cmd = self.to_cmd_and_addr(cmd, false);
        self.inner.read_data(&cmd, params, buffer).await
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        self.inner.set_reset(reset)
    }
}

/// A wrapper that implements efficient solid filling by batching writes into a larger buffer.
///
/// This is useful when the underlying bus (e.g., `SimpleDisplayBus`) has a small default
/// buffer or high per-transaction overhead.
pub struct BatchFillBus<B: DisplayBus, const N: usize> {
    inner: B,
    strict_metadata: bool,
}

impl<B: DisplayBus, const N: usize> BatchFillBus<B, N> {
    /// Creates a new BatchFillBus.
    ///
    /// `strict_metadata` controls how the fill area is split:
    /// - `true`: Splits are aligned to rows or sub-rows, ensuring metadata `w` and `h` accurately reflect the data chunk.
    /// - `false`: Behavior depends on implementation details, but generally assumes stricter adherence isn't required.
    pub fn new(inner: B, strict_metadata: bool) -> Self {
        Self { inner, strict_metadata }
    }
}

impl<B: DisplayBus, const N: usize> DisplayBus for BatchFillBus<B, N> {
    type Error = B::Error;

    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        self.inner.write_cmds(cmd).await
    }

    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error> {
        self.inner.write_cmd_with_params(cmd, params).await
    }

    async fn write_pixels(&mut self, cmd: &[u8], data: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        self.inner.write_pixels(cmd, data, metadata).await
    }

    async fn fill_solid(&mut self, cmd: &[u8], color: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        let pixel_size = color.len();
        if pixel_size == 0 {
            return Ok(());
        }
        
        // Calculate the maximum number of full pixels that fit in the buffer.
        let max_pixels_per_batch = N / pixel_size;
        if max_pixels_per_batch == 0 {
             return Err(DisplayError::InvalidArgs);
        }

        // Prepare the fill buffer once.
        let mut buffer = [0u8; N];
        for i in 0..max_pixels_per_batch {
            buffer[i * pixel_size..(i + 1) * pixel_size].copy_from_slice(color);
        }

        if self.strict_metadata {
            // STRICT MODE:
            // Ensure every transaction represents a valid rectangular area (x, y, w, h).
            // Useful for controllers that require precise window setting for every data burst.
            
            let mut current_y = metadata.y;
            let mut rows_remaining = metadata.h;
            let bytes_per_row = metadata.w as usize * pixel_size;

            while rows_remaining > 0 {
                if bytes_per_row <= N {
                    // Scenario 1: One or more FULL rows fit in the buffer.
                    // We can batch multiple rows vertically.
                    let rows_fit_in_buffer = N / bytes_per_row;
                    let rows_to_send = rows_remaining.min(rows_fit_in_buffer as u16);
                    
                    let batch_bytes = rows_to_send as usize * bytes_per_row;
                    let sub_meta = Metadata {
                        x: metadata.x,
                        y: current_y,
                        w: metadata.w,
                        h: rows_to_send,
                    };

                    self.inner.write_pixels(cmd, &buffer[..batch_bytes], sub_meta).await?;

                    current_y += rows_to_send;
                    rows_remaining -= rows_to_send;
                } else {
                    // Scenario 2: A single row is too large for the buffer.
                    // We must split the row horizontally into smaller chunks.
                    let mut current_x = metadata.x;
                    let mut cols_remaining = metadata.w;

                    while cols_remaining > 0 {
                        let cols_to_send = cols_remaining.min(max_pixels_per_batch as u16);
                        let batch_bytes = cols_to_send as usize * pixel_size;

                        let sub_meta = Metadata {
                            x: current_x,
                            y: current_y,
                            w: cols_to_send,
                            h: 1, // Single line height
                        };

                        self.inner.write_pixels(cmd, &buffer[..batch_bytes], sub_meta).await?;

                        current_x += cols_to_send;
                        cols_remaining -= cols_to_send;
                    }

                    current_y += 1;
                    rows_remaining -= 1;
                }
            }
        } else {
            // NON-STRICT MODE:
            // Ignore geometric rows; treat the area as a linear stream of pixels.
            // This maximizes throughput by always filling the buffer, relying on the 
            // display hardware's address pointer to auto-wrap to the next line.
            
            let total_pixels = metadata.w as usize * metadata.h as usize;
            let mut pixels_remaining = total_pixels;
            
            while pixels_remaining > 0 {
                let pixels_to_send = pixels_remaining.min(max_pixels_per_batch);
                let batch_bytes = pixels_to_send * pixel_size;

                // Note: x/y are generally ignored in this mode by the controller if 
                // it's in continuous write mode, but we pass valid-ish data.
                let sub_meta = Metadata {
                    x: metadata.x,
                    y: metadata.y,
                    w: pixels_to_send as u16,
                    h: 1,
                };

                self.inner.write_pixels(cmd, &buffer[..batch_bytes], sub_meta).await?;

                pixels_remaining -= pixels_to_send;
            }
        }
        
        Ok(())
    }

    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        self.inner.read_data(cmd, params, buffer).await
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        self.inner.set_reset(reset)
    }
}