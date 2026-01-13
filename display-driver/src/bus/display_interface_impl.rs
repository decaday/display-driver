//! Implementation of `SimpleDisplayBus` for `display-interface` traits.

use display_interface::{AsyncWriteOnlyDataCommand, DisplayError, DataFormat};
use super::SimpleDisplayBus;

impl<DI: AsyncWriteOnlyDataCommand> SimpleDisplayBus for DI {
    type Error = DisplayError;

    // fn configure(&mut self, config: Config) -> Result<(), crate::DisplayError<Self::Error>> {
    //     if config.cmd_size_bytes != 1 {
    //         todo!()
    //     } else {
    //         Ok(())
    //     }
    // }
    
    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        self.send_commands(DataFormat::U8(cmd)).await
    }
    
    async fn write_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.send_data(DataFormat::U8(data)).await
    }
}

