#[cfg(feature = "display-interface")]
mod display_interface_impl;

pub mod qspi_flash;
pub use qspi_flash::QspiFlashBus;

pub mod simple;
pub use simple::SimpleDisplayBus;

use crate::{Area, DisplayError, SolidColor};

/// Error type trait.
///
/// This just defines the error type, to be used by the other traits.
pub trait ErrorType {
    /// Error type
    type Error: core::fmt::Debug;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FrameControl {
    pub first: bool,
    pub last: bool,
}

impl FrameControl {
    pub fn new_standalone() -> Self {
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
///
/// Advanced display buses (like MIPI DSI or QSPI with DMA) often require more context than just the
/// raw pixel bytes.
/// This struct carries that side-band information, allowing the bus implementation to orchestrate 
/// the transfer correctly.
#[derive(Clone, Copy, Debug)]
pub struct Metadata {
    /// The rectangular area on the display this data corresponds to.
    ///
    /// If `Some`, the bus may use this to set the active window before sending data.
    /// If `None`, the data is assumed to be a continuation of the previous stream.
    pub area: Option<Area>,
    /// Flags for frame synchronization (start/end of frame).
    pub frame_control: FrameControl,
}

impl Metadata {
    /// Creates metadata for a full screen update.
    ///
    /// This sets the area to the full display dimensions and marks the transfer as both the start
    /// and end of a frame.
    /// Use this for standard full-frame refreshing.
    pub fn new_full_screen(w: u16, h: u16) -> Self {
        Self {
            area: Some(Area::from_origin(w, h)),
            frame_control: FrameControl {
                first: true,
                last: true,
            },
        }
    }

    /// Creates metadata for continuing a stream of pixel data without resetting the area or frame
    /// markers.
    ///
    /// Use this when splitting a large frame into multiple smaller chunks for transfer.
    pub fn new_continue_stream() -> Self {
        Self {
            area: None,
            frame_control: FrameControl {
                first: false,
                last: false,
            },
        }
    }

    /// Creates metadata with specific area and frame control settings.
    ///
    /// Use this for partial updates or specialized transfer patterns.
    pub fn new_from_parts(area: Option<Area>, frame_control: FrameControl) -> Self {
        Self {
            area,
            frame_control,
        }
    }
}

#[allow(async_fn_in_trait)]
/// The core interface for all display bus implementations.
///
/// This trait serves as the abstraction layer between the high-level drawing logic and the
/// low-level transport protocol. It accommodates a wide range of hardware, from simple 2-wire
/// interfaces to complex high-speed buses.
///
/// The interface distinguishes between two types of traffic:
/// - **Commands**: Small, latency-sensitive messages used for configuration (handled by `write_cmd`
///   and `write_cmd_with_params`).
/// - **Pixels**: Large, throughput-critical data streams used for changing the visual content 
///   (handled by `write_pixels`).
///
/// This separation allows for optimizations. For instance, `write_pixels` accepts [`Metadata`], 
/// enabling the underlying implementation to utilize hardware accelerators (like DMA or QSPI 
/// peripherals) that can handle address setting and bulk data transfer efficiently.
pub trait DisplayBus: ErrorType {
    /// Writes a command to the display.
    ///
    /// This is typically used for setting registers or sending configuration opcodes.
    async fn write_cmd(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    // async fn write_cmds(&mut self, cmds: &[u8]) -> Result<(), Self::Error>;

    /// Writes a command followed immediately by its parameters.
    ///
    /// This guarantees an atomic transaction where the command and parameters are sent without 
    /// interruption. This is critical for many display controllers that expect the parameter bytes
    /// to immediately follow the command byte while the Chip Select (CS) line remains active.
    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8])
        -> Result<(), Self::Error>;

    /// Writes a stream of pixel data to the display.
    ///
    /// # Arguments
    /// * `cmd` - The memory write command (e.g., `0x2C` for standard MIPI DCS).
    /// * `data` - The raw pixel data bytes.
    /// * `metadata` - Contextual information about this transfer, including the target area and
    ///   frame boundaries.
    ///
    /// Implementations should use the `metadata` to handle frame synchronization (VSYNC/TE) before
    /// sending the pixel data.
    async fn write_pixels(
        &mut self,
        cmd: &[u8],
        data: &[u8],
        metadata: Metadata,
    ) -> Result<(), DisplayError<Self::Error>>;

    /// Resets the screen via the bus (optional).
    /// 
    /// Note: This method should only be implemented if the hardware has a physical Reset pin.
    /// Avoid adding a Pin field to your `DisplayBus` wrapper for this purpose; use `LCDResetOption`
    /// instead.
    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        let _ = reset;
        Err(DisplayError::Unsupported)
    }
}

#[allow(async_fn_in_trait)]
/// An optional trait for buses that support hardware-accelerated solid color filling.
///
/// Filling a large area with a single color is a common operation (e.g., clearing the screen).
/// If the hardware supports it (e.g., via a 2D GPU or a DMA channel with a non-incrementing source
/// address), this trait allows the driver to offload that work, significantly reducing CPU usage
/// and bus traffic.
pub trait BusHardwareFill: DisplayBus {
    /// Fills a specific region of the display with a solid color.
    ///
    /// The implementation should leverage available hardware acceleration to perform this operation
    /// efficiently.
    async fn fill_solid(
        &mut self,
        cmd: &[u8],
        color: SolidColor,
        area: Area,
    ) -> Result<(), DisplayError<Self::Error>>;
}

#[allow(async_fn_in_trait)]
/// An optional trait for buses that support reading data back from the display.
///
/// While most display interactions are write-only, reading is sometimes necessary for:
/// - Verifying the connection by reading the display ID.
/// - Checking status registers.
/// - Reading back frame memory (e.g., for screenshots), though this is less common.
///
/// Not all physical interfaces support bi-directional communication (e.g., SPI TFT is often 
/// write-only).
pub trait BusRead: DisplayBus {
    /// Reads data from the display.
    ///
    /// # Arguments
    /// * `cmd` - The command to initiate the read operation.
    /// * `params` - Optional parameters required before the read transaction begins.
    /// * `buffer` - The destination buffer where the read data will be stored.
    async fn read_data(
        &mut self,
        cmd: &[u8],
        params: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), DisplayError<Self::Error>> {
        let (_, _, _) = (cmd, params, buffer);
        Err(DisplayError::Unsupported)
    }
}

/// An optional trait for buses that support non-atomic command and data writing.
///
/// Some buses, such as SPI, support sending commands and data in a single transaction, while others
/// require separate transactions for commands and data.
#[allow(async_fn_in_trait)]
pub trait BusBytesIo: DisplayBus {
    /// Writes a sequence of commands to the bus.
    ///
    /// This is typically used for sending register addresses or command opcodes.
    async fn write_cmd_bytes(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    /// Writes a sequence of data bytes to the bus.
    ///
    /// This is used for sending command parameters or pixel data.
    async fn write_data_bytes(&mut self, data: &[u8]) -> Result<(), Self::Error>;
}
