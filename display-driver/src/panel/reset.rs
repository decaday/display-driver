use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use crate::{DisplayBus, DisplayError};

/// Option for LCD reset control.
pub enum LCDResetOption<P: OutputPin> {
    /// Reset via a GPIO pin (active high).
    PinHigh(P),
    /// Reset via a GPIO pin (active low).
    PinLow(P),
    /// Reset via the display bus.
    Bus,
    /// Reset via software command (e.g. 0x01).
    Software,
    /// No hardware reset.
    None,
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
            Self::Software => None,
            Self::None => None,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl LCDResetOption<NoResetPin> {
    /// Creates a Bus reset option.
    pub fn new_bus() -> Self {
        Self::Bus
    }

    /// Creates a Software reset option.
    pub fn new_software() -> Self {
        Self::Software
    }

    /// Creates a None reset option.
    pub fn none() -> Self {
        Self::None
    }
}

/// Helper to handle LCD hardware reset.
pub struct LCDResetHandler<'a, P: OutputPin, B: DisplayBus, D: DelayNs> {
    option: &'a mut LCDResetOption<P>,
    bus: &'a mut B,
    delay: &'a mut D,
    gap_ms: u8,
    wait_ms: u8,
    software_reset_cmd: Option<&'a [u8]>,
}

impl<'a, P: OutputPin, B: DisplayBus, D: DelayNs> LCDResetHandler<'a, P, B, D> {
    /// Creates a new LCDResetHandler.
    pub fn new(
        option: &'a mut LCDResetOption<P>,
        bus: &'a mut B,
        delay: &'a mut D,
        gap_ms: u8,
        wait_ms: u8,
        software_reset_cmd: Option<&'a [u8]>,
    ) -> Self {
        Self {
            option,
            bus,
            delay,
            gap_ms,
            wait_ms,
            software_reset_cmd,
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
            }
            LCDResetOption::PinLow(ref mut pin) => {
                if reset {
                    pin.set_low().map_err(|_| unreachable!())
                } else {
                    pin.set_high().map_err(|_| unreachable!())
                }
            }
            LCDResetOption::Bus => self.bus.set_reset(reset).map_err(|err| match err {
                DisplayError::BusError(e) => e,
                DisplayError::Unsupported => panic!("Bus cannot reset"),
                _ => unreachable!(),
            }),
            LCDResetOption::Software => Ok(()),
            LCDResetOption::None => unreachable!(),
        }
    }

    /// Performs the reset sequence: assert -> wait -> release -> wait.
    pub async fn reset(&mut self) -> Result<(), B::Error> {
        if matches!(self.option, LCDResetOption::Software) {
            if let Some(cmd) = self.software_reset_cmd {
                self.bus.write_cmd(cmd).await?;
                self.delay.delay_ms(self.wait_ms as u32).await;
            }
            return Ok(());
        }

        if !self.option.is_none() {
            self.set_reset(false)?;
            self.delay.delay_ms(self.gap_ms as u32).await;
            self.set_reset(true)?;
            self.delay.delay_ms(self.gap_ms as u32).await;
            self.set_reset(false)?;
            self.delay.delay_ms(self.wait_ms as u32).await;
        }
        Ok(())
    }
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
