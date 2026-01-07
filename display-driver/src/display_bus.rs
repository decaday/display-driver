use crate::{DisplayError, ColorFormat};

pub struct Config {
    pub screen_width: u16,
    pub screen_height: u16,

    pub pixel_bpp: u8,
    pub color_format: ColorFormat,
    pub cmd_size_bytes: u8,
    // msb: bool
}

#[allow(async_fn_in_trait)]
pub trait SimpleDisplayBus {
    type Error;

    // fn configure(&mut self, config: Config) -> Result<(), DisplayError<Self::Error>> {
    //     let _ = config;
    //     Ok(())
    // }

    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    async fn write_data(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error> {
        self.write_cmds(cmd).await?;
        self.write_data(params).await
    }

    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        let (_, _, _) = (cmd, params, buffer);
        Err(DisplayError::Unsupported)
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        let _ = reset;
        Err(DisplayError::Unsupported)
    }
}

pub struct Metadata {
    pub width: u16,
    pub height: u16,
}

#[allow(async_fn_in_trait)]
pub trait DisplayBus {
    type Error;

    // fn configure(&mut self, config: Config) -> Result<(), DisplayError<Self::Error>>;

    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error>;

    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error>;

    async fn write_pixels(&mut self, cmd: &[u8], params: &[u8], buffer: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>>;

    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        let (_, _, _) = (cmd, params, buffer);
        Err(DisplayError::Unsupported)
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        let _ = reset;
        Err(DisplayError::Unsupported)
    }
}

impl<T: SimpleDisplayBus> DisplayBus for T {
    type Error = T::Error;

    // fn configure(&mut self, config: Config) -> Result<(), DisplayError<Self::Error>> {
    //     T::configure(self, config)
    // }

    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        T::write_cmds(self, cmd).await
    }

    async fn write_cmd_with_params(&mut self, cmd: &[u8], params: &[u8]) -> Result<(), Self::Error> {
        T::write_cmd_with_params(self, cmd, params).await
    }

    async fn write_pixels(&mut self, cmd: &[u8], params: &[u8], buffer: &[u8], _metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        self.write_cmds(cmd).await.map_err(DisplayError::BusError)?;
        self.write_data(params).await.map_err(DisplayError::BusError)?;
        self.write_data(buffer).await.map_err(DisplayError::BusError)
    }

    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        T::read_data(self, cmd, params, buffer).await
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        T::set_reset(self, reset)
    }
}

pub struct QspiFlashBus<DB: DisplayBus> {
    inner: DB,
}

impl<DB: DisplayBus> QspiFlashBus<DB> {
    pub fn new(inner: DB) -> Self {
        Self { inner }
    }

    pub fn to_cmd_and_addr(&self, cmd: &[u8], pixel_data: bool) -> [u8; 4] {
        if cmd.len() != 1 {
            panic!()
        }

        let flash_command: u8 = if pixel_data {
            0x32
        } else {
            0x02
        };

        [flash_command, 0x00, cmd[0], 0x00]
    }
}

impl<DB: DisplayBus> DisplayBus for QspiFlashBus<DB> {
    type Error = DB::Error;

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

    async fn write_pixels(&mut self, cmd: &[u8], params: &[u8], buffer: &[u8], metadata: Metadata) -> Result<(), DisplayError<Self::Error>> {
        let cmd = self.to_cmd_and_addr(cmd, true);
        self.inner.write_pixels(&cmd, params, buffer, metadata).await
    }

    async fn read_data(&mut self, cmd: &[u8], params: &[u8], buffer: &mut [u8]) -> Result<(), DisplayError<Self::Error>> {
        let cmd = self.to_cmd_and_addr(cmd, false);
        self.inner.read_data(&cmd, params, buffer).await
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        self.inner.set_reset(reset)
    }
}