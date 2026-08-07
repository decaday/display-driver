#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use display_driver::bus::DisplayBus;
use display_driver::panel::initseq::{sequenced_init, InitStep};
use display_driver::panel::reset::{LCDResetHandler, LCDResetOption};
use display_driver::panel::{Orientation, Panel, PanelSetBrightness};

use display_driver::{ColorFormat, DisplayError};

use display_driver_mipidcs as mipidcs;
use display_driver_mipidcs::{AddressMode, GenericMipidcs};

pub mod consts;
pub mod spec;

use consts::*;
pub use spec::{St77916Spec, NT150XV};

/// Driver for the ST77916 display controller.
pub struct St77916<Spec, RST, B>
where
    Spec: St77916Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    /// Inner generic driver for standard functionality.
    inner: GenericMipidcs<B, Spec, RST>,
}

impl<Spec, RST, B> St77916<Spec, RST, B>
where
    Spec: St77916Spec,
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

    const INIT_STEPS: &'static [InitStep<'static>] = &[
        InitStep::CommandWithParams(CSC1, &[0x28]),
        InitStep::CommandWithParams(CSC3, &[0x28]),
        InitStep::CommandWithParams(0x73, &[0xF0]),
        InitStep::CommandWithParams(0x7C, &[0xD1]),
        InitStep::CommandWithParams(0x83, &[0xE0]),
        InitStep::CommandWithParams(0x84, &[0x61]),
        InitStep::CommandWithParams(CSC3, &[0x82]),
        InitStep::CommandWithParams(CSC1, &[0x00]),

        // Command2 Page Enable
        InitStep::CommandWithParams(CSC1, &[0x01]),
        InitStep::CommandWithParams(CSC2, &[0x01]),
        InitStep::Nested(Spec::COMMAND2_PAGE),

        // Gamma Page Enable
        InitStep::CommandWithParams(CSC2, &[0x10]),
        InitStep::CommandWithParams(CSC1, &[0x00]),
        InitStep::CommandWithParams(CSC1, &[0x02]),
        InitStep::CommandWithParams(GAMCTRP1, &Spec::GAMCTRP1_PARAMS),
        InitStep::CommandWithParams(GAMCTRN1, &Spec::GAMCTRN1_PARAMS),

        // GIP Command Page Enable
        InitStep::CommandWithParams(CSC1, &[0x10]),
        InitStep::CommandWithParams(CSC4, &[0x10]),
        InitStep::Nested(Spec::GIP_PAGE),

        // Disable pages
        InitStep::CommandWithParams(CSC4, &[0x01]),
        InitStep::CommandWithParams(CSC1, &[0x00]),
        InitStep::CommandWithParams(mipidcs::SET_PIXEL_FORMAT, &[0x05]),
        // InitStep::CommandWithParams(mipidcs::SET_TEAR_ON, &[0x00]),
        InitStep::select_cmd(
            Spec::INVERTED,
            mipidcs::ENTER_INVERT_MODE,
            mipidcs::EXIT_INVERT_MODE,
        ),
        InitStep::CommandWithParams(
            mipidcs::SET_ADDRESS_MODE,
            &[if Spec::BGR {
                AddressMode::BGR.bits()
            } else {
                0u8
            }],
        ),
        InitStep::SingleCommand(mipidcs::EXIT_SLEEP_MODE),
        InitStep::DelayMs(120),
        InitStep::SingleCommand(mipidcs::SET_DISPLAY_ON),
    ];
}

impl<Spec, RST, B> Panel<B> for St77916<Spec, RST, B>
where
    Spec: St77916Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    const CMD_LEN: usize = 1;
    const PIXEL_WRITE_CMD: [u8; 4] = [mipidcs::WRITE_MEMORY_START, 0, 0, 0];

    async fn init<D: DelayNs>(&mut self, bus: &mut B, mut delay: D) -> Result<(), B::Error> {
        // Hardware Reset
        let mut reseter = LCDResetHandler::new(
            &mut self.inner.reset_pin,
            bus,
            &mut delay,
            10,
            120,
            Some(&[mipidcs::SOFT_RESET]),
        );
        reseter.reset().await?;

        // Execute Initialization Sequence
        // copied() only copies the items during iteration; it does not copy the entire sequence
        sequenced_init(Self::INIT_STEPS.iter().copied(), &mut delay, bus).await
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

impl<Spec, RST, B> PanelSetBrightness<B> for St77916<Spec, RST, B>
where
    Spec: St77916Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    async fn set_brightness(
        &mut self,
        bus: &mut B,
        brightness: u8,
    ) -> Result<(), DisplayError<B::Error>> {
        bus.write_cmd_with_params(&[WRDISBV], &[brightness])
            .await
            .map_err(DisplayError::BusError)
    }
}
