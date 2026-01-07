// #![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use crate::display_bus::{DisplayBus, Flags};
use crate::panel::{sequenced_init, InitStep, LCDResetOption, LCDReseter, Panel};

pub mod consts;
pub mod spec;

use spec::DisplaySpec;
use consts::*;

/// CO5300 LCD Driver.
pub struct Co5300<Spec, RST>
where
    RST: OutputPin,
{
    reset_pin: LCDResetOption<RST>,
    _spec: core::marker::PhantomData<Spec>,
}

impl<Spec, RST> Co5300<Spec, RST>
where
    Spec: DisplaySpec,
    RST: OutputPin,
{
    /// Creates a new driver instance.
    pub fn new(reset_pin: LCDResetOption<RST>) -> Self {
        Self {
            reset_pin,
            _spec: core::marker::PhantomData,
        }
    }

    /// Sets the display brightness (0-255).
    pub async fn set_brightness<B>(&mut self, bus: &mut B, value: u8) -> Result<(), B::Error>
    where
        B: DisplayBus,
    {
        bus.write_cmd_with_params(&[WBRIGHT], Flags::default(), &[value])
            .await
    }

    /// Sets the color mode (RGB565 or RGB888).
    pub async fn set_color_mode<B>(&mut self, bus: &mut B, mode: ColorMode) -> Result<(), B::Error>
    where
        B: DisplayBus,
    {
        bus.write_cmd_with_params(&[COLOR_MODE], Flags::default(), &[mode as u8])
            .await
    }
}

impl<Spec, RST, B> Panel<B> for Co5300<Spec, RST>
where
    Spec: DisplaySpec,
    RST: OutputPin,
    B: DisplayBus,
{
    async fn init<D: DelayNs>(&mut self, bus: &mut B, mut delay: D) -> Result<(), B::Error> {
        // 1. Hardware Reset
        let mut reseter = LCDReseter::new(&mut self.reset_pin, bus, &mut delay, 10);
        reseter.reset().await?;

        // 3. Define Initialization Sequence
        let steps = [
            // Unlock Sequence
            InitStep::CommandWithParams((CMD_PAGE_SWITCH, &[Spec::INIT_PAGE_PARAM])),
            InitStep::CommandWithParams((PASSWD1, &[0x5A])),
            InitStep::CommandWithParams((PASSWD2, &[0x59])),
            // Lock Sequence
            InitStep::CommandWithParams((CMD_PAGE_SWITCH, &[0x20])),
            InitStep::CommandWithParams((PASSWD1, &[0xA5])),
            InitStep::CommandWithParams((PASSWD2, &[0xA5])),
            // Configuration
            InitStep::CommandWithParams((CMD_PAGE_SWITCH, &[0x00])),
            InitStep::CommandWithParams((SPI_MODE, &[0x80])),
            InitStep::CommandWithParams((COLOR_MODE, &[0x55])),
            InitStep::CommandWithParams((TEARING_EFFECT_ON, &[0x00])),
            InitStep::CommandWithParams((WRITE_CTRL_DISPLAY, &[0x20])),
            InitStep::CommandWithParams((WRHBMDISBV, &[0xFF])),
            // Address Window Setup
            // InitStep::CommandWithParams((CASET, &x_buf)),
            // InitStep::CommandWithParams((RASET, &y_buf)),
            // Power On
            InitStep::SingleCommand(SLEEP_OUT),
            InitStep::DelayMs(120),
            InitStep::SingleCommand(DISPLAY_ON),
            InitStep::DelayMs(70),
        ];

        // 4. Execute Sequence
        sequenced_init(steps.into_iter(), &mut delay, bus, Flags::default()).await
    }

    fn size(&self) -> (u16, u16) {
        (Spec::WIDTH, Spec::HEIGHT)
    }

    async fn set_window(
        &mut self,
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), B::Error> {
        let x_start = x0 + Spec::COL_OFFSET;
        let x_end = x1 + Spec::COL_OFFSET;
        let y_start = y0 + Spec::ROW_OFFSET;
        let y_end = y1 + Spec::ROW_OFFSET;

        let x_buf = [
            (x_start >> 8) as u8,
            (x_start & 0xFF) as u8,
            (x_end >> 8) as u8,
            (x_end & 0xFF) as u8,
        ];

        let y_buf = [
            (y_start >> 8) as u8,
            (y_start & 0xFF) as u8,
            (y_end >> 8) as u8,
            (y_end & 0xFF) as u8,
        ];

        let flags = Flags::default();
        bus.write_cmd_with_params(&[CASET], flags, &x_buf).await?;
        bus.write_cmd_with_params(&[RASET], flags, &y_buf).await?;

        Ok(())
    }

    async fn start_write_pixels(&mut self, bus: &mut B) -> Result<(), B::Error> {
        let flags = Flags::default().with_bulk(true);
        bus.write_cmd(&[WRITE_RAM], flags, true).await
    }
}