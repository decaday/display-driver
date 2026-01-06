use embedded_hal_async::delay::DelayNs;

use crate::display_bus::{DisplayBus, Flags};

pub enum PanelError<E> {
    BusError(E),
    Unsupported,
}

pub enum Orientation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[allow(async_fn_in_trait)]
pub trait Panel<B: DisplayBus> {
    async fn init(&mut self, bus: &mut B) -> Result<(), B::Error>;

    async fn set_window(&mut self, 
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), PanelError<B::Error>>;

    async fn start_write_pixels(&mut self, 
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), B::Error>;

    async fn end_write_pixels(&mut self, 
        bus: &mut B,
    ) -> Result<(), B::Error> {
        bus.end_write().await
    }

    async fn verify_id(&mut self, 
        bus: &mut B,
    ) -> Result<bool, PanelError<B::Error>> {
        let _ = bus;
        Err(PanelError::Unsupported)
    }

    async fn set_orientation(&mut self, 
        bus: &mut B,
        orientation: Orientation,
    ) -> Result<(), PanelError<B::Error>> {
        let _ = (bus, orientation);
        Err(PanelError::Unsupported)
    }

    // async fn set_rgb_order(&mut self, 
    //     bus: &mut B,
    //     rgb_order: bool,
    // ) -> Result<(), PanelError<B::Error>> {
    //     let _ = (bus, rgb_order);
    //     Err(PanelError::Unsupported)
    // }
}

pub enum InitStep<'a> {
    SingleCommand(u8),
    CommandWithParams((u8, &'a [u8])),
    DelayMs(u8),
}

pub struct SequencedInit<'a, D: DelayNs, DB: DisplayBus, I: Iterator<Item = InitStep<'a>>> {
    steps: I,
    delay: &'a mut D,
    display_bus: &'a mut DB,
    flags: Flags,
}

impl<'a, D: DelayNs, DB: DisplayBus, I: Iterator<Item = InitStep<'a>>> SequencedInit<'a, D, DB, I> {
    pub fn new(steps: I, delay: &'a mut D, display_bus: &'a mut DB, flags: Flags) -> Self {
        Self {
            steps,
            delay,
            display_bus,
            flags,
        }
    }

    pub async fn sequenced_init(&mut self) -> Result<(), DB::Error> {
        while let Some(step) = self.steps.next() {
            match step {
                InitStep::SingleCommand(cmd) => {
                    let flags = self.flags.with_continuous(false);
                    self.display_bus.write_cmd(&[cmd], flags).await?
                },
                InitStep::CommandWithParams((cmd, data)) => {
                    self.display_bus.write_cmd_with_params(&[cmd], self.flags, data).await?
                },
                InitStep::DelayMs(ms) => self.delay.delay_ms(ms as u32).await,
            }
        }

        Ok(())
    }
}

pub async fn sequenced_init<'a, D: DelayNs, DB: DisplayBus, I: Iterator<Item = InitStep<'a>>>(steps: I, delay: &'a mut D, display_bus: &'a mut DB, flags: Flags) -> Result<(), DB::Error> {
    SequencedInit::new(steps, delay, display_bus, flags).sequenced_init().await
}