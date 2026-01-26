use super::{DisplayBus, Metadata, DisplayError, ErrorType};

/// An adapter that bridges a standard [`DisplayBus`] to a QSPI-connected display.
///
/// Some displays attached via QSPI don't use standard display commands directly but instead emulate
/// a SPI Flash memory interface. This adapter wraps an inner bus and automatically transforms
/// standard display commands into the appropriate QSPI Flash opcodes.
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

impl<B: DisplayBus> ErrorType for QspiFlashBus<B> {
    type Error = B::Error;
}

impl<B: DisplayBus> DisplayBus for QspiFlashBus<B> {

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

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        self.inner.set_reset(reset)
    }
}