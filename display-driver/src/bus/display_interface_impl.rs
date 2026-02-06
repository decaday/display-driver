//! Implementation of `SimpleDisplayBus` for `display-interface` traits.

use super::{ErrorType, SimpleDisplayBus};
use display_interface::{AsyncWriteOnlyDataCommand, DataFormat, DisplayError};

impl<DI: AsyncWriteOnlyDataCommand> ErrorType for DI {
    type Error = DisplayError;
}

impl<DI: AsyncWriteOnlyDataCommand> SimpleDisplayBus for DI {
    async fn write_cmds(&mut self, cmds: &[u8]) -> Result<(), Self::Error> {
        self.send_commands(DataFormat::U8(cmds)).await
    }

    async fn write_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.send_data(DataFormat::U8(data)).await
    }
}
