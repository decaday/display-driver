use display_interface::{AsyncWriteOnlyDataCommand, DisplayError, DataFormat};
use crate::display_bus::{DisplayBus, Flags};

impl<DI: AsyncWriteOnlyDataCommand + Send> DisplayBus for DI {
    type Error = DisplayError;

    async fn write_cmd(&mut self, cmd: &[u8], flags: Flags) -> Result<(), Self::Error> {
        match cmd.len() {
            1 => {
                self.send_commands(DataFormat::U8(cmd)).await
            }
            2 => {
                let u16cmd = u16::from_be_bytes([cmd[0], cmd[1]]);
                let cmd = if flags.le() {
                        DataFormat::U16LE(&mut [u16cmd])
                } else {
                        DataFormat::U16BE(&mut [u16cmd])
                };
                self.send_commands(cmd).await
            }
            _ => {
                Err(DisplayError::InvalidFormatError)
            }
        }
    }

    async fn write_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.send_data(DataFormat::U8(data)).await
    }
}