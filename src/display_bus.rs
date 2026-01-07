use bitfield_struct::bitfield;

#[bitfield(u32)]
pub struct Flags {
    #[bits(1, default = false)]
    pub lsb_first: bool,
    #[bits(1, default = false)]
    pub le: bool,
    #[bits(1, default = false)]
    pub bulk: bool,
    #[bits(29)]
    reserved: u32,
}

#[allow(async_fn_in_trait)]
pub trait DisplayBus {
    type Error;

    async fn write_cmd(&mut self, cmd: &[u8], flags: Flags, continuous: bool) -> Result<(), Self::Error>;

    async fn write_data(&mut self, data: &[u8], continuous: bool) -> Result<(), Self::Error>;

    async fn write_cmd_with_params(&mut self, cmd: &[u8], flags: Flags, params: &[u8]) -> Result<(), Self::Error> {
        self.write_cmd(cmd, flags, true).await?;
        self.write_data(params, false).await
    }

    async fn write_pixels(&mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        pixels: &[u8],
    ) -> Result<(), Self::Error> {
        let (_, _ , _ , _ ) = (x0, y0, x1, y1);
        self.write_data(pixels, false).await
    }

    async fn read_data(&mut self, cmd: &[u8], flags: Flags, buffer: &mut [u8]) -> Option<Result<(), Self::Error>> {
        let (_, _, _) = (cmd, flags, buffer);
        None
    }

    fn set_reset(&mut self, reset: bool) -> Option<Result<(), Self::Error>> {
        let _ = reset;
        None
    }
}

pub struct QspiMmioBus<DB: DisplayBus> {
    inner: DB
}
impl<DB: DisplayBus> QspiMmioBus<DB> {
    pub fn new(inner: DB) -> Self {
        Self { inner }
    }

    pub fn to_cmd_and_addr(&self, cmd: &[u8], flags: Flags) -> [u8; 4] {
        if cmd.len() != 1 {
            panic!()
        }

        let flash_command: u8 = if flags.bulk() {
            0x32
        } else {
            0x02
        };

        [flash_command, 0x00, cmd[0], 0x00]
    }
}

impl<DB: DisplayBus> DisplayBus for QspiMmioBus<DB> {
    type Error = DB::Error;

    async fn write_cmd(&mut self, cmd: &[u8], flags: Flags, continuous: bool) -> Result<(), Self::Error> {
        let cmd = self.to_cmd_and_addr(cmd, flags);
        self.inner.write_cmd(&cmd, flags, continuous).await
    }

    async fn write_data(&mut self, data: &[u8], continuous: bool) -> Result<(), Self::Error> {
        self.inner.write_data(data, continuous).await
    }

    async fn write_cmd_with_params(&mut self, cmd: &[u8], flags: Flags, params: &[u8]) -> Result<(), Self::Error> {
        let cmd = self.to_cmd_and_addr(cmd, flags);
        self.inner.write_cmd_with_params(&cmd, flags, params).await
    }

    async fn write_pixels(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, pixels: &[u8]) -> Result<(), Self::Error> {
        self.inner.write_pixels(x0, y0, x1, y1, pixels).await
    }

    async fn read_data(&mut self, cmd: &[u8], flags: Flags, buffer: &mut [u8]) -> Option<Result<(), Self::Error>> {
        let cmd = self.to_cmd_and_addr(cmd, flags);
        self.inner.read_data(&cmd, flags, buffer).await
    }
}