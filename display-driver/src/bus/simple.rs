use super::*;

#[allow(async_fn_in_trait)]
/// A simplified interface for display buses that don't need atomic command with params, complex frame
/// control, ROI information, etc. (e.g., standard SPI, I2C).
///
/// This trait abstracts over simple serial interfaces where commands and data are just streams of bytes.
/// It provides a convenient way to implement the full [`DisplayBus`] trait without worrying about
/// frame metadata or pixel-specific handling, as those are handled by the blanket implementation.
///
/// Implementors only need to define how to send raw command bytes and raw data bytes.
pub trait SimpleDisplayBus: ErrorType {
    /// Writes a sequence of commands to the bus.
    ///
    /// This is typically used for sending register addresses or command opcodes.
    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    /// Writes a sequence of data bytes to the bus.
    ///
    /// This is used for sending command parameters or pixel data.
    async fn write_data(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Writes a command followed immediately by its parameters.
    async fn write_cmd_with_params(
        &mut self,
        cmd: &[u8],
        params: &[u8],
    ) -> Result<(), Self::Error> {
        self.write_cmds(cmd).await?;
        self.write_data(params).await
    }

    /// Reset the screen via the bus (optional).
    /// Note: This method should only be implemented if the hardware has a physical Reset pin.
    /// Avoid adding a Pin field to your `DisplayBus` wrapper for this purpose; use `LCDResetOption` instead.
    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        let _ = reset;
        Err(DisplayError::Unsupported)
    }
}

impl<T: SimpleDisplayBus> BusNonAtomicCmdData for T {
    async fn write_cmds_non_atomic(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        T::write_cmds(self, cmd).await
    }

    async fn write_data_non_atomic(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        T::write_data(self, data).await
    }
}

impl<T: SimpleDisplayBus> DisplayBus for T {
    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        T::write_cmds(self, cmd).await
    }

    async fn write_cmd_with_params(
        &mut self,
        cmd: &[u8],
        params: &[u8],
    ) -> Result<(), Self::Error> {
        T::write_cmd_with_params(self, cmd, params).await
    }

    async fn write_pixels(
        &mut self,
        cmd: &[u8],
        data: &[u8],
        _metadata: Metadata,
    ) -> Result<(), DisplayError<Self::Error>> {
        T::write_cmd_with_params(self, cmd, data)
            .await
            .map_err(DisplayError::BusError)
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        T::set_reset(self, reset)
    }
}
