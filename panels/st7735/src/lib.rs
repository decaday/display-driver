#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use display_driver::bus::DisplayBus;
use display_driver::panel::initseq::{sequenced_init, InitStep};
use display_driver::panel::reset::{LCDResetOption, LCDReseter};
use display_driver::panel::{Orientation, Panel};

use display_driver::{ColorFormat, DisplayError};

use mipidcs::SET_ADDRESS_MODE;
use mipidcs::{dcs_types::AddressMode, GenericMipidcs};

pub mod consts;
pub mod spec;

use consts::*;
use spec::St7735Spec;

/// Driver for the ST7735 display controller.
pub struct St7735<Spec, RST, B>
where
    Spec: St7735Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    /// Inner generic driver for standard functionality.
    inner: GenericMipidcs<B, Spec, RST>,
}

impl<Spec, RST, B> St7735<Spec, RST, B>
where
    Spec: St7735Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    /// Creates a new driver instance.
    pub fn new(reset_pin: LCDResetOption<RST>) -> Self {
        Self {
            inner: GenericMipidcs::new(reset_pin),
        }
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

    /// Initialization sequence for ST7735.
    const INIT_STEPS: [InitStep<'static>; 18] = [
        // Sleep Out
        InitStep::SingleCommand(mipidcs::EXIT_SLEEP_MODE),
        InitStep::DelayMs(120),
        // Frame Rate Configuration
        InitStep::CommandWithParams((FRMCTR1, &Spec::FRMCTR1_PARAMS)),
        InitStep::CommandWithParams((FRMCTR2, &Spec::FRMCTR2_PARAMS)),
        InitStep::CommandWithParams((FRMCTR3, &Spec::FRMCTR3_PARAMS)),
        InitStep::CommandWithParams((INVCTR, &[Spec::INVCTR_PARAM])),
        // Power Configuration
        InitStep::CommandWithParams((PWCTR1, &Spec::PWCTR1_PARAMS)),
        InitStep::CommandWithParams((PWCTR2, &[Spec::PWCTR2_PARAM])),
        InitStep::CommandWithParams((PWCTR3, &Spec::PWCTR3_PARAMS)),
        InitStep::CommandWithParams((PWCTR4, &Spec::PWCTR4_PARAMS)),
        InitStep::CommandWithParams((PWCTR5, &Spec::PWCTR5_PARAMS)),
        InitStep::CommandWithParams((VMCTR1, &[Spec::VMCTR1_PARAM])),
        // Invert mode
        InitStep::select_cmd(
            Spec::INVERTED,
            mipidcs::ENTER_INVERT_MODE,
            mipidcs::EXIT_INVERT_MODE,
        ),
        InitStep::CommandWithParams((
            SET_ADDRESS_MODE,
            &[if Spec::BGR {
                AddressMode::BGR.bits()
            } else {
                0u8
            }],
        )),
        // Gamma Correction
        InitStep::maybe_cmd_with(GMCTRP1, Spec::GMCTRP1_PARAMS),
        InitStep::maybe_cmd_with(GMCTRN1, Spec::GMCTRN1_PARAMS),
        // Display On
        InitStep::SingleCommand(mipidcs::SET_DISPLAY_ON),
        InitStep::DelayMs(20), // Small delay after turning on
    ];
}

impl<Spec, RST, B> Panel<B> for St7735<Spec, RST, B>
where
    Spec: St7735Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    const CMD_LEN: usize = 1;
    const PIXEL_WRITE_CMD: [u8; 4] = [mipidcs::WRITE_MEMORY_START, 0, 0, 0];

    async fn init<D: DelayNs>(&mut self, bus: &mut B, mut delay: D) -> Result<(), B::Error> {
        // Hardware Reset
        let mut reseter = LCDReseter::new(
            &mut self.inner.reset_pin,
            bus,
            &mut delay,
            10,
            120,
            Some(&[mipidcs::SOFT_RESET]),
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
