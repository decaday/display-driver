

/// Line mode for a specific phase of a QSPI transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineMode {
    /// Phase is not present
    None,
    /// Single-line (1-bit) mode
    Single,
    /// Dual-line (2-bit) mode
    Dual,
    /// Quad-line (4-bit) mode
    Quad,
}

/// Configuration for a specific phase (Instruction or Address) in a QSPI transaction.
#[derive(Debug, Clone, Copy)]
pub struct PhaseConfig {
    /// The value to transmit in this phase
    pub value: u32,
    /// The number of bytes to transmit (e.g., 1 for an 8-bit instruction, 3 for a 24-bit address)
    pub bytes_len: u8,
    /// The line mode used for this phase
    pub mode: LineMode,
}

/// A QSPI transaction descriptor.
#[derive(Debug, Clone, Copy)]
pub struct QspiTransaction {
    /// The instruction phase configuration, if any
    pub instruction: Option<PhaseConfig>,
    /// The address phase configuration, if any
    pub address: Option<PhaseConfig>,
    /// The number of dummy cycles to wait before the data phase
    pub dummy_cycles: u8,
    /// The line mode used for the data phase
    pub data_mode: LineMode,
}

/// Hardware abstraction trait for QSPI devices.
///
/// MCU-specific HAL wrappers should implement this trait so that the `QspiDisplayBus`
/// can interact with the hardware over QSPI.
#[allow(async_fn_in_trait)]
pub trait QspiDevice {
    /// Error type for the QSPI operations.
    type Error: core::fmt::Debug;

    /// Perform a QSPI write operation.
    ///
    /// The transaction specifies the instruction, address, dummy cycles, and the data phase mode.
    /// The `data` slice contains the payload to write during the data phase.
    async fn write(
        &mut self,
        transaction: &QspiTransaction,
        data: &[u8],
    ) -> Result<(), Self::Error>;

    /// Perform a QSPI read operation.
    ///
    /// The transaction specifies the instruction, address, dummy cycles, and the data phase mode.
    /// The `buffer` slice will be filled with the data read during the data phase.
    async fn read(
        &mut self,
        transaction: &QspiTransaction,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error>;
}
