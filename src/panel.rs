use embedded_hal_async::delay::DelayNs;
use embedded_hal::digital::OutputPin;

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
    async fn init<D: DelayNs>(&mut self, bus: &mut B, delay: D) -> Result<(), B::Error>;

    fn size(&self) -> (u16, u16);

    // fn offset(&self) -> (u16, u16);

    async fn set_window(&mut self, 
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), B::Error>;

    async fn start_write_pixels(&mut self, 
        bus: &mut B,
    ) -> Result<(), B::Error>;

    async fn end_write_pixels(&mut self, 
        bus: &mut B,
    ) -> Result<(), B::Error> { Ok(()) }

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
                    self.display_bus.write_cmd(&[cmd], self.flags, false).await?
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


pub struct NoResetPin {}
impl embedded_hal::digital::ErrorType for NoResetPin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for NoResetPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(PartialEq, Eq)]
pub enum LCDResetOption<P: OutputPin> {
    PinHigh(P),
    PinLow(P),
    Bus,
    None
}

impl<P: OutputPin> LCDResetOption<P> {
    pub fn new_pin(pin: P) -> Self {
        Self::PinLow(pin)
    }

    pub fn new_pin_with_level(pin: P, reset_level: bool) -> Self {
        if reset_level {
            Self::PinHigh(pin)
        } else {
            Self::PinLow(pin)
        }
    }

    pub fn release(self) -> Option<P> {
        match self {
            Self::PinHigh(pin) => Some(pin),
            Self::PinLow(pin) => Some(pin),
            Self::Bus => None,
            Self::None => None,
        }
    }
}

impl LCDResetOption<NoResetPin> {
    pub fn new_bus() -> Self {
        Self::Bus
    }

    pub fn none() -> Self {
        Self::None
    }
}


pub struct LCDReseter<'a, P: OutputPin, DB: DisplayBus, D: DelayNs> {
    option: &'a mut LCDResetOption<P>,
    bus: &'a mut DB,
    delay: &'a mut D,
    gap_ms: u8,
}

impl<'a, P: OutputPin, DB: DisplayBus, D: DelayNs> LCDReseter<'a, P, DB, D> {
    pub fn new(option: &'a mut LCDResetOption<P>, bus: &'a mut DB, delay: &'a mut D, gap_ms: u8) -> Self {
        Self {
            option,
            bus,
            delay,
            gap_ms
        }
    }

    pub fn set_reset(&mut self, reset: bool) -> Result<(), DB::Error> {
        match *self.option {
            LCDResetOption::PinHigh(ref mut pin) => {
                if reset {
                    pin.set_high().map_err(|_| unreachable!())
                } else {
                    pin.set_low().map_err(|_| unreachable!())
                }
            },
            LCDResetOption::PinLow(ref mut pin) => {
                if reset {
                    pin.set_low().map_err(|_| unreachable!())
                } else {
                    pin.set_high().map_err(|_| unreachable!())
                }
            },
            LCDResetOption::Bus => {
                self.bus.set_reset(reset).expect("Bus cannot reset")
            },
            LCDResetOption::None => Ok(()),
        }
    }

    pub async fn reset(&mut self) -> Result<(), DB::Error> {
        self.set_reset(false)?;
        self.delay.delay_ms(self.gap_ms as u32).await;
        self.set_reset(true)?;
        self.delay.delay_ms(self.gap_ms as u32).await;
        self.set_reset(false)?;
        self.delay.delay_ms(self.gap_ms as u32).await;

        Ok(())
    }
}
