use embedded_hal_async::delay::DelayNs;
use embedded_hal::digital::OutputPin;

use crate::{DisplayError, ColorFormat, DisplayBus};

/// Display orientation.
pub enum Orientation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[allow(async_fn_in_trait)]
/// Trait for display panels.
pub trait Panel<B: DisplayBus> {
    /// Initializes the panel.
    async fn init<D: DelayNs>(&mut self, bus: &mut B, delay: D) -> Result<(), B::Error>;

    /// Returns the panel resolution (width, height).
    fn size(&self) -> (u16, u16);

    // fn offset(&self) -> (u16, u16);

    /// Sets the active window for pixel writing.
    async fn set_window(&mut self, 
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), DisplayError<B::Error>>;

    /// Sets the window to the full screen size.
    async fn set_full_window(&mut self, bus: &mut B) -> Result<(), DisplayError<B::Error>> {
        let (w, h) = self.size();
        self.set_window(bus, 0, 0, w - 1, h - 1).await
    }

    /// Writes pixels to the specified area.
    /// 
    /// # Arguments
    /// * `bus` - The display bus interface.
    /// * `x` - Start X coordinate.
    /// * `y` - Start Y coordinate.
    /// * `w` - Width of the area.
    /// * `h` - Height of the area.
    /// * `buffer` - Pixel data.
    async fn write_pixels(&mut self, 
        bus: &mut B,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        buffer: &[u8],
    ) -> Result<(), DisplayError<B::Error>>;

    async fn fill_solid(&mut self, 
        bus: &mut B,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        color: &[u8],
    ) -> Result<(), DisplayError<B::Error>>;

    /// Fills the entire screen with a solid color.
    async fn fill_screen(&mut self, bus: &mut B, color: &[u8]) -> Result<(), DisplayError<B::Error>> {
        let (w, h) = self.size();
        self.fill_solid(bus, 0, 0, w - 1, h - 1, color).await
    }

    /// Verifies the panel ID (if supported).
    async fn verify_id(&mut self, 
        bus: &mut B,
    ) -> Result<bool, DisplayError<B::Error>> {
        let _ = bus;
        Err(DisplayError::Unsupported)
    }

    /// Sets the display orientation.
    async fn set_orientation(&mut self, 
        bus: &mut B,
        orientation: Orientation,
    ) -> Result<(), DisplayError<B::Error>> {
        let _ = (bus, orientation);
        Err(DisplayError::Unsupported)
    }

    /// Sets the color format.
    async fn set_color_format(&mut self, 
        bus: &mut B,
        color_format: ColorFormat,
    ) -> Result<(), DisplayError<B::Error>>;

    /// Sets the brightness (0-255).
    async fn set_brightness(&mut self, 
        bus: &mut B,
        brightness: u8,
    ) -> Result<(), DisplayError<B::Error>> {
        let _ = (bus, brightness);
        Err(DisplayError::Unsupported)
    }

    // async fn set_rgb_order(&mut self, 
    //     bus: &mut B,
    //     rgb_order: bool,
    // ) -> Result<(), DisplayError<B::Error>> {
    //     let _ = (bus, rgb_order);
    //     Err(DisplayError::Unsupported)
    // }
}

/// A step in the initialization sequence.
#[derive(Clone, Copy)] 
pub enum InitStep<'a> {
    /// Single byte command.
    SingleCommand(u8),
    /// Command with parameters.
    CommandWithParams((u8, &'a [u8])),
    /// Delay in milliseconds.
    DelayMs(u8),
    /// No Operation. Useful for placeholders or conditional logic.
    Nop,
    /// Nested sequence. 
    /// NOTE: Only supports one level of nesting (cannot nest a Nested inside a Nested)
    /// to avoid recursion issues in async no_std environments.
    Nested(&'a [InitStep<'a>]),
}

/// Helper to execute initialization steps.
pub struct SequencedInit<'a, D: DelayNs, B: DisplayBus, I: Iterator<Item = InitStep<'a>>> {
    steps: I,
    delay: &'a mut D,
    display_bus: &'a mut B,
}

impl<'a, D: DelayNs, B: DisplayBus, I: Iterator<Item = InitStep<'a>>> SequencedInit<'a, D, B, I> {
    /// Creates a new SequencedInit instance.
    pub fn new(steps: I, delay: &'a mut D, display_bus: &'a mut B) -> Self {
        Self {
            steps,
            delay,
            display_bus,
        }
    }

    /// Helper function to execute a single atomic step.
    /// Does not handle recursion; ignores Nested variants if passed directly.
    async fn exec_atomic_step(&mut self, step: InitStep<'a>) -> Result<(), B::Error> {
        match step {
            InitStep::SingleCommand(cmd) => {
                self.display_bus.write_cmds(&[cmd]).await
            },
            InitStep::CommandWithParams((cmd, data)) => {
                self.display_bus.write_cmd_with_params(&[cmd], data).await
            },
            InitStep::DelayMs(ms) => {
                self.delay.delay_ms(ms as u32).await;
                Ok(())
            },
            InitStep::Nop => Ok(()),
            InitStep::Nested(_) => {
                panic!("We only support 1 level in InitStep.");
            }
        }
    }

    /// Executes the initialization sequence.
    pub async fn sequenced_init(&mut self) -> Result<(), B::Error> {
        while let Some(step) = self.steps.next() {
            match step {
                // If the step is a Nested sequence, unroll it here (1 level deep)
                InitStep::Nested(sub_steps) => {
                    for sub_step in sub_steps.iter() {
                        // We use *sub_step because we derived Copy
                        self.exec_atomic_step(*sub_step).await?;
                    }
                },
                // Otherwise, execute the step directly
                _ => self.exec_atomic_step(step).await?,
            }
        }

        Ok(())
    }
}

/// Convenience function to run an initialization sequence.
pub async fn sequenced_init<'a, D: DelayNs, B: DisplayBus, I: Iterator<Item = InitStep<'a>>>(steps: I, delay: &'a mut D, display_bus: &'a mut B) -> Result<(), B::Error> {
    SequencedInit::new(steps, delay, display_bus).sequenced_init().await
}


/// Dummy pin implementation for when no reset pin is used.
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
/// Option for LCD reset control.
pub enum LCDResetOption<P: OutputPin> {
    /// Reset via a GPIO pin (active high).
    PinHigh(P),
    /// Reset via a GPIO pin (active low).
    PinLow(P),
    /// Reset via the display bus.
    Bus,
    /// No hardware reset.
    None
}

impl<P: OutputPin> LCDResetOption<P> {
    /// Creates a new PinLow reset option.
    pub fn new_pin(pin: P) -> Self {
        Self::PinLow(pin)
    }

    /// Creates a new reset option with specified active level.
    pub fn new_pin_with_level(pin: P, reset_level: bool) -> Self {
        if reset_level {
            Self::PinHigh(pin)
        } else {
            Self::PinLow(pin)
        }
    }

    /// Releases the pin if held.
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
    /// Creates a Bus reset option.
    pub fn new_bus() -> Self {
        Self::Bus
    }

    /// Creates a None reset option.
    pub fn none() -> Self {
        Self::None
    }
}


/// Helper to handle LCD hardware reset.
pub struct LCDReseter<'a, P: OutputPin, B: DisplayBus, D: DelayNs> {
    option: &'a mut LCDResetOption<P>,
    bus: &'a mut B,
    delay: &'a mut D,
    gap_ms: u8,
}

impl<'a, P: OutputPin, B: DisplayBus, D: DelayNs> LCDReseter<'a, P, B, D> {
    /// Creates a new LCDReseter.
    pub fn new(option: &'a mut LCDResetOption<P>, bus: &'a mut B, delay: &'a mut D, gap_ms: u8) -> Self {
        Self {
            option,
            bus,
            delay,
            gap_ms
        }
    }

    /// Sets the reset state.
    pub fn set_reset(&mut self, reset: bool) -> Result<(), B::Error> {
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
                self.bus.set_reset(reset).map_err(|err| match err {
                    DisplayError::BusError(e) => e,
                    DisplayError::Unsupported => panic!("Bus cannot reset"),
                    _ => unreachable!(),
                }) 
            },
            LCDResetOption::None => Ok(()),
        }
    }

    /// Performs the reset sequence: assert -> wait -> release -> wait.
    pub async fn reset(&mut self) -> Result<(), B::Error> {
        self.set_reset(false)?;
        self.delay.delay_ms(self.gap_ms as u32).await;
        self.set_reset(true)?;
        self.delay.delay_ms(self.gap_ms as u32).await;
        self.set_reset(false)?;
        self.delay.delay_ms(self.gap_ms as u32).await;

        Ok(())
    }
}

pub const fn address_window_param_u8(start: u16, end: u16, offset: u16) -> [u8; 4] {
        let s = (start + offset).to_be_bytes();
        let e = (end + offset).to_be_bytes();
        [s[0], s[1], e[0], e[1]]
}