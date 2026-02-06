use crate::{Area, SolidColor};

use super::{BusHardwareFill, BusRead, DisplayBus, DisplayError, ErrorType, Metadata};

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

    #[inline]
    fn assert_cmd_len(&self, cmd: &[u8]) {
        assert_eq!(
            cmd.len(),
            1,
            "QspiFlashBus only supports single byte commands"
        );
    }

    /// Formats the command and address for QSPI transfer.
    /// This is used for common write commands except WRITE_RAM.
    #[inline]
    pub fn to_cmd_and_addr_command(&self, cmd: u8) -> [u8; 4] {
        [0x02, 0x00, cmd, 0x00]
    }

    /// Formats the command and address for QSPI transfer.
    /// This is used for WRITE_RAM command.
    #[inline]
    pub fn to_cmd_and_addr_write_ram(&self, cmd: u8) -> [u8; 4] {
        [0x32, 0x00, cmd, 0x00]
    }

    /// Formats the command and address for QSPI transfer.
    /// This is used for read commands.
    #[inline]
    pub fn to_cmd_and_addr_read(&self, cmd: u8) -> [u8; 4] {
        [0x03, 0x00, cmd, 0x00]
    }
}

impl<B: DisplayBus> ErrorType for QspiFlashBus<B> {
    type Error = B::Error;
}

impl<B: DisplayBus> DisplayBus for QspiFlashBus<B> {
    async fn write_cmd(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        self.assert_cmd_len(cmd);
        let cmd = self.to_cmd_and_addr_command(cmd[0]);
        self.inner.write_cmd(&cmd).await
    }

    async fn write_cmd_with_params(
        &mut self,
        cmd: &[u8],
        params: &[u8],
    ) -> Result<(), Self::Error> {
        self.assert_cmd_len(cmd);
        let cmd = self.to_cmd_and_addr_command(cmd[0]);
        self.inner.write_cmd_with_params(&cmd, params).await
    }

    async fn write_pixels(
        &mut self,
        cmd: &[u8],
        data: &[u8],
        metadata: Metadata,
    ) -> Result<(), DisplayError<Self::Error>> {
        self.assert_cmd_len(cmd);
        let cmd = self.to_cmd_and_addr_write_ram(cmd[0]);
        self.inner.write_pixels(&cmd, data, metadata).await
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        self.inner.set_reset(reset)
    }
}

impl<B: DisplayBus + BusHardwareFill> BusHardwareFill for QspiFlashBus<B> {
    async fn fill_solid(
        &mut self,
        cmd: &[u8],
        color: SolidColor,
        area: Area,
    ) -> Result<(), DisplayError<Self::Error>> {
        self.assert_cmd_len(cmd);
        let cmd = self.to_cmd_and_addr_write_ram(cmd[0]);
        self.inner.fill_solid(&cmd, color, area).await
    }
}

impl<B: DisplayBus + BusRead> BusRead for QspiFlashBus<B> {
    async fn read_data(
        &mut self,
        cmd: &[u8],
        params: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), DisplayError<Self::Error>> {
        self.assert_cmd_len(cmd);
        let cmd = self.to_cmd_and_addr_read(cmd[0]);
        self.inner.read_data(&cmd, params, buffer).await
    }
}
