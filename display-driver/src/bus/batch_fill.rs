use super::{DisplayBus, Metadata, SingleColor, DisplayError};

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

    async fn fill_solid(&mut self, cmd: &[u8], color: SingleColor, metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        let pixel_size = color.format.size_bytes() as usize;
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
        let color_bytes = &color.raw[..pixel_size];
        
        for i in 0..max_pixels_per_batch {
            buffer[i * pixel_size..(i + 1) * pixel_size].copy_from_slice(color_bytes);
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
