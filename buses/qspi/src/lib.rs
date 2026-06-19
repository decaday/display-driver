#![no_std]

pub mod hal;

pub use hal::{LineMode, PhaseConfig, QspiDevice, QspiTransaction};

use display_driver::{
    bus::{DisplayBus, ErrorType, Metadata},
    DisplayError,
};

const WRITE_CMD_INST: u8 = 0x02;
const WRITE_RAM_INST: u8 = 0x32;
const READ_INST: u8 = 0x03;

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

    /// Helper to generate a 24-bit address with the DCS command in the middle byte.
    /// Example: cmd `0x2C` becomes `0x002C00`.
    #[inline(always)]
    fn make_address(cmd: u8) -> PhaseConfig {
        PhaseConfig {
            value: (cmd as u32) << 8,
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

        let transaction = QspiTransaction {
            instruction: Some(PhaseConfig {
                value: WRITE_CMD_INST as u32,
                bytes_len: 1,
                mode: self.config.cmd_data_mode,
            }),
            address: Some(Self::make_address(cmd[0])),
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

        let transaction = QspiTransaction {
            instruction: Some(PhaseConfig {
                value: WRITE_CMD_INST as u32,
                bytes_len: 1,
                mode: self.config.cmd_data_mode,
            }),
            address: Some(Self::make_address(cmd[0])),
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

        let transaction = QspiTransaction {
            instruction: Some(PhaseConfig {
                value: WRITE_RAM_INST as u32,
                bytes_len: 1,
                mode: self.config.cmd_data_mode,
            }),
            address: Some(Self::make_address(cmd[0])),
            dummy_cycles: self.config.dummy_cycles,
            data_mode: self.config.ram_data_mode,
        };

        self.qspi.write(&transaction, data).await.map_err(DisplayError::BusError)
    }
}

impl<QSPI> display_driver::bus::BusRead for QspiDisplayBus<QSPI>
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

        let transaction = QspiTransaction {
            instruction: Some(PhaseConfig {
                value: READ_INST as u32,
                bytes_len: 1,
                mode: self.config.cmd_data_mode,
            }),
            address: Some(Self::make_address(cmd[0])),
            dummy_cycles: self.config.dummy_cycles,
            data_mode: self.config.cmd_data_mode,
        };

        self.qspi.read(&transaction, buffer).await.map_err(DisplayError::BusError)
    }
}
