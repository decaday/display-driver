#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use display_driver::bus::DisplayBus;
use display_driver::panel::initseq::{sequenced_init, InitStep};
use display_driver::panel::reset::{LCDResetOption, LCDReseter};
use display_driver::panel::{Orientation, Panel};

use display_driver::{ColorFormat, DisplayError};

// Use GenericMipidcs to handle standard DCS operations
use mipidcs::{consts::*, dcs_types::AddressMode, GenericMipidcs};

pub mod consts;
pub mod spec;

use consts::*;
use spec::Co5300Spec;

/// Driver for the CO5300 AMOLED display controller.
pub struct Co5300<Spec, RST, B>
where
    Spec: Co5300Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    /// Inner generic driver for standard functionality.
    inner: GenericMipidcs<B, Spec, RST>,
}

impl<Spec, RST, B> Co5300<Spec, RST, B>
where
    Spec: Co5300Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    /// Creates a new driver instance.
    pub fn new(reset_pin: LCDResetOption<RST>) -> Self {
        Self {
            inner: GenericMipidcs::new(reset_pin),
        }
    }

    /// Sets the display brightness (0-255).
    pub async fn set_brightness(
        &mut self,
        bus: &mut B,
        value: u8,
    ) -> Result<(), DisplayError<B::Error>> {
        bus.write_cmd_with_params(&[WBRIGHT], &[value])
            .await
            .map_err(DisplayError::BusError)
    }

    delegate::delegate! {
        to self.inner {
            pub async fn set_invert_mode(
                &mut self,
                bus: &mut B,
                invert: bool,
            ) -> Result<(), B::Error>;

            pub async fn set_address_mode(
                &mut self,
                bus: &mut B,
                address_mode: AddressMode,
                orientation_if_changed: Option<Orientation>,
            ) -> Result<(), B::Error>;

            pub async fn set_bgr_order(&mut self, bus: &mut B, bgr: bool) -> Result<(), B::Error>;
        }
    }

    /// Initialization sequence for CO5300.
    const INIT_STEPS: [InitStep<'static>; 16] = [
        // Unlock Sequence
        InitStep::CommandWithParams(CMD_PAGE_SWITCH, &[Spec::INIT_PAGE_PARAM]),
        InitStep::CommandWithParams(PASSWD1, &[0x5A]),
        InitStep::CommandWithParams(PASSWD2, &[0x59]),
        // Lock Sequence
        InitStep::CommandWithParams(CMD_PAGE_SWITCH, &[0x20]),
        InitStep::CommandWithParams(PASSWD1, &[0xA5]),
        InitStep::CommandWithParams(PASSWD2, &[0xA5]),
        // Configuration
        InitStep::CommandWithParams(CMD_PAGE_SWITCH, &[0x00]),
        InitStep::CommandWithParams(SPI_MODE, &[0x80]),
        InitStep::CommandWithParams(COLOR_MODE, &[0x55]), // Default to RGB565
        InitStep::CommandWithParams(TEARING_EFFECT_ON, &[0x00]),
        InitStep::CommandWithParams(WRITE_CTRL_DISPLAY, &[0x20]),
        InitStep::CommandWithParams(WRHBMDISBV, &[0xFF]),
        // Power On
        InitStep::SingleCommand(SLEEP_OUT),
        InitStep::DelayMs(120),
        InitStep::SingleCommand(DISPLAY_ON),
        InitStep::DelayMs(70),
    ];
}

impl<Spec, RST, B> Panel<B> for Co5300<Spec, RST, B>
where
    Spec: Co5300Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    const CMD_LEN: usize = 1;
    const PIXEL_WRITE_CMD: [u8; 4] = [WRITE_RAM, 0, 0, 0];

    fn x_alignment(&self) -> u16 {
        2
    }

    fn y_alignment(&self) -> u16 {
        2
    }

    async fn init<D: DelayNs>(&mut self, bus: &mut B, mut delay: D) -> Result<(), B::Error> {
        let mut reseter = LCDReseter::new(
            &mut self.inner.reset_pin,
            bus,
            &mut delay,
            10,
            120,
            Some(&[SOFT_RESET]),
        );
        reseter.reset().await?;

        // Execute Initialization Sequence
        sequenced_init(Self::INIT_STEPS.into_iter(), &mut delay, bus).await
    }

    delegate::delegate! {
        to self.inner {
            fn width(&self) -> u16;

            fn height(&self) -> u16;

            fn size(&self) -> (u16, u16);

            async fn set_window(
                &mut self,
                bus: &mut B,
                x0: u16,
                y0: u16,
                x1: u16,
                y1: u16,
            ) -> Result<(), DisplayError<B::Error>>;

            async fn set_color_format(
                &mut self,
                bus: &mut B,
                color_format: ColorFormat,
            ) -> Result<(), DisplayError<B::Error>>;

            async fn set_orientation(
                &mut self,
                bus: &mut B,
                orientation: Orientation,
            ) -> Result<(), DisplayError<B::Error>>;
        }
    }
}
