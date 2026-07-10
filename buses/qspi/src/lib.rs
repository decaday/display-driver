#![no_std]

pub mod hal;

pub use hal::{LineMode, PhaseConfig, QspiDevice, QspiTransaction};

use display_driver::{
    bus::{BusHardwareFill, BusRead, DisplayBus, ErrorType, Metadata},
    Area, DisplayError, SolidColor,
};

const WRITE_CMD_INST: u8 = 0x02;
const WRITE_RAM_INST: u8 = 0x32;
const READ_INST: u8 = 0x03;

/// QSPI flash command mapping.
/// It encapsulates the mapping from a 1-byte DCS command to a QSPI flash instruction
/// and address.
#[derive(Debug, Clone, Copy)]
pub struct QspiFlashCommand {
    pub inst: u8,
    pub cmd: u8,
}

impl QspiFlashCommand {
    /// Create a new QSPI flash command.
    pub fn new(inst: u8, cmd: u8) -> Self {
        Self { inst, cmd }
    }

    /// Convert the command to a 4-byte sequence: [instruction, 0x00, command, 0x00]
    pub fn to_bytes(&self) -> [u8; 4] {
        [self.inst, 0x00, self.cmd, 0x00]
    }

    /// Get the 24-bit address value.
    pub fn address_value(&self) -> u32 {
        (self.cmd as u32) << 8
    }
}

/// Configuration for the QSPI display bus.
///
/// This specifies the line modes to use for different phases
/// of communication with the display.
#[derive(Debug, Clone, Copy)]
pub struct QspiConfig {
    /// The line mode used for transmitting parameter data during command read/write operations.
    ///
    /// Selecting `LineMode::Quad` represents the 1-1-4 mode (1-bit instruction, 1-bit address, 4-bit data).
    pub cmd_data_mode: LineMode,
    /// The line mode used for transmitting pixel data.
    ///
    /// Selecting `LineMode::Quad` represents the 1-1-4 mode (1-bit instruction, 1-bit address, 4-bit data).
    pub ram_data_mode: LineMode,
    /// Number of dummy cycles required before reading or writing data (if any).
    pub dummy_cycles: u8,
}

impl Default for QspiConfig {
    fn default() -> Self {
        Self {
            cmd_data_mode: LineMode::Single,
            ram_data_mode: LineMode::Quad,
            dummy_cycles: 0,
        }
    }
}

/// A QSPI bus implementation for display drivers.
///
/// **TODO: Protocol Strategy Abstraction**
/// Currently, this bus implementation uses a fixed, hardcoded QSPI protocol sequence.
/// It hardcodes both the QSPI instructions (e.g., `0x02` for command write, `0x32` for RAM write)
/// and the address-mapping strategy (packing a 1-byte MIPI DCS command into the middle byte of
/// a 24-bit address phase, such as `0x00_CMD_00`).
/// While this specific protocol works out-of-the-box for many display drivers like CO5300 and ST77916,
/// it is incompatible with others like ST77903.
/// As we integrate and test more diverse QSPI protocols, we should introduce a dedicated
/// Trait to support customizable and pluggable command/address generation strategies.
pub struct QspiDisplayBus<QSPI> {
    qspi: QSPI,
    config: QspiConfig,
}

impl<QSPI> QspiDisplayBus<QSPI> {
    /// Creates a new QspiDisplayBus with the given hardware device and configuration.
    pub fn new(qspi: QSPI, config: QspiConfig) -> Self {
        Self { qspi, config }
    }

    /// Helper to generate a 24-bit address using the shared QspiFlashCommand.
    #[inline(always)]
    fn make_address(cmd: &QspiFlashCommand) -> PhaseConfig {
        PhaseConfig {
            value: cmd.address_value(),
            bytes_len: 3,
            mode: LineMode::Single,
        }
    }

    /// Verifies that the command slice contains exactly one byte.
    #[inline(always)]
    fn assert_cmd_len(cmd: &[u8]) {
        assert_eq!(
            cmd.len(),
            1,
            "QspiDisplayBus currently only supports single byte DCS commands"
        );
    }
}

impl<QSPI> ErrorType for QspiDisplayBus<QSPI>
where
    QSPI: QspiDevice,
{
    type Error = QSPI::Error;
}

impl<QSPI> DisplayBus for QspiDisplayBus<QSPI>
where
    QSPI: QspiDevice,
{
    async fn write_cmd(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        Self::assert_cmd_len(cmd);
        let flash_cmd = QspiFlashCommand::new(WRITE_CMD_INST, cmd[0]);

        let transaction = QspiTransaction {
            instruction: Some(PhaseConfig {
                value: flash_cmd.inst as u32,
                bytes_len: 1,
                mode: self.config.cmd_data_mode,
            }),
            address: Some(Self::make_address(&flash_cmd)),
            dummy_cycles: 0,
            data_mode: LineMode::None,
        };

        self.qspi.write(&transaction, &[]).await
    }

    async fn write_cmd_with_params(
        &mut self,
        cmd: &[u8],
        params: &[u8],
    ) -> Result<(), Self::Error> {
        Self::assert_cmd_len(cmd);
        let flash_cmd = QspiFlashCommand::new(WRITE_CMD_INST, cmd[0]);

        let transaction = QspiTransaction {
            instruction: Some(PhaseConfig {
                value: flash_cmd.inst as u32,
                bytes_len: 1,
                mode: self.config.cmd_data_mode,
            }),
            address: Some(Self::make_address(&flash_cmd)),
            dummy_cycles: 0,
            data_mode: self.config.cmd_data_mode,
        };

        self.qspi.write(&transaction, params).await
    }

    async fn write_pixels(
        &mut self,
        cmd: &[u8],
        data: &[u8],
        _metadata: Metadata,
    ) -> Result<(), DisplayError<Self::Error>> {
        Self::assert_cmd_len(cmd);
        let flash_cmd = QspiFlashCommand::new(WRITE_RAM_INST, cmd[0]);

        let transaction = QspiTransaction {
            instruction: Some(PhaseConfig {
                value: flash_cmd.inst as u32,
                bytes_len: 1,
                mode: self.config.cmd_data_mode,
            }),
            address: Some(Self::make_address(&flash_cmd)),
            dummy_cycles: self.config.dummy_cycles,
            data_mode: self.config.ram_data_mode,
        };

        self.qspi.write(&transaction, data).await.map_err(DisplayError::BusError)
    }
}

impl<QSPI> BusRead for QspiDisplayBus<QSPI>
where
    QSPI: QspiDevice,
{
    async fn read_data(
        &mut self,
        cmd: &[u8],
        _params: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), DisplayError<Self::Error>> {
        Self::assert_cmd_len(cmd);
        let flash_cmd = QspiFlashCommand::new(READ_INST, cmd[0]);

        let transaction = QspiTransaction {
            instruction: Some(PhaseConfig {
                value: flash_cmd.inst as u32,
                bytes_len: 1,
                mode: self.config.cmd_data_mode,
            }),
            address: Some(Self::make_address(&flash_cmd)),
            dummy_cycles: self.config.dummy_cycles,
            data_mode: self.config.cmd_data_mode,
        };

        self.qspi.read(&transaction, buffer).await.map_err(DisplayError::BusError)
    }
}

/// A Decorator that wraps an inner `DisplayBus` and translates 1-byte DCS commands
/// into 4-byte QSPI flash commands (instruction + 3-byte address).
///
/// This provides the same mapping effect as `QspiDisplayBus` but runs on top of
/// any generic `DisplayBus` implementation, useful for hardware that does not
/// implement the `QspiDevice` Trait.
pub struct QspiFlashBus<B> {
    inner: B,
}

impl<B> QspiFlashBus<B> {
    /// Creates a new `QspiFlashBus` wrapping the given inner bus.
    pub fn new(inner: B) -> Self {
        Self { inner }
    }

    #[inline(always)]
    fn assert_cmd_len(cmd: &[u8]) {
        assert_eq!(
            cmd.len(),
            1,
            "QspiFlashBus currently only supports single byte DCS commands"
        );
    }
}

impl<B: ErrorType> ErrorType for QspiFlashBus<B> {
    type Error = B::Error;
}

impl<B: DisplayBus> DisplayBus for QspiFlashBus<B> {
    async fn write_cmd(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        Self::assert_cmd_len(cmd);
        let flash_cmd = QspiFlashCommand::new(WRITE_CMD_INST, cmd[0]);
        self.inner.write_cmd(&flash_cmd.to_bytes()).await
    }

    async fn write_cmd_with_params(
        &mut self,
        cmd: &[u8],
        params: &[u8],
    ) -> Result<(), Self::Error> {
        Self::assert_cmd_len(cmd);
        let flash_cmd = QspiFlashCommand::new(WRITE_CMD_INST, cmd[0]);
        self.inner.write_cmd_with_params(&flash_cmd.to_bytes(), params).await
    }

    async fn write_pixels(
        &mut self,
        cmd: &[u8],
        data: &[u8],
        metadata: Metadata,
    ) -> Result<(), DisplayError<Self::Error>> {
        Self::assert_cmd_len(cmd);
        let flash_cmd = QspiFlashCommand::new(WRITE_RAM_INST, cmd[0]);
        self.inner.write_pixels(&flash_cmd.to_bytes(), data, metadata).await
    }
}

impl<B: BusRead> BusRead for QspiFlashBus<B> {
    async fn read_data(
        &mut self,
        cmd: &[u8],
        params: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), DisplayError<Self::Error>> {
        Self::assert_cmd_len(cmd);
        let flash_cmd = QspiFlashCommand::new(READ_INST, cmd[0]);
        self.inner.read_data(&flash_cmd.to_bytes(), params, buffer).await
    }
}

impl<B: BusHardwareFill> BusHardwareFill for QspiFlashBus<B> {
    async fn fill_solid(
        &mut self,
        cmd: &[u8],
        color: SolidColor,
        area: Area,
    ) -> Result<(), DisplayError<Self::Error>> {
        Self::assert_cmd_len(cmd);
        let flash_cmd = QspiFlashCommand::new(WRITE_RAM_INST, cmd[0]);
        self.inner.fill_solid(&flash_cmd.to_bytes(), color, area).await
    }
}
