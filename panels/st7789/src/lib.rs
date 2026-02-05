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
pub use spec::St7789Spec;

/// Driver for the ST7789 display controller.
pub struct St7789<Spec, RST, B>
where
    Spec: St7789Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    /// Inner generic driver for standard functionality.
    inner: GenericMipidcs<B, Spec, RST>,
}

impl<Spec, RST, B> St7789<Spec, RST, B>
where
    Spec: St7789Spec,
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

    /// Initialization sequence for ST7789.
    const INIT_STEPS: [InitStep<'static>; 21] = [
        // Sleep Out
        InitStep::SingleCommand(mipidcs::EXIT_SLEEP_MODE),
        InitStep::DelayMs(120),
        // Interface Pixel Format
        InitStep::CommandWithParams(
            mipidcs::SET_PIXEL_FORMAT,
            &mipidcs::PixelFormat::dbi_and_dpi(mipidcs::PixelFormatType::Bits16).as_bytes(),
        ),
        // Porch Setting
        InitStep::CommandWithParams(PORCTRL, &Spec::PORCTRL_PARAMS),
        // Gate Control
        InitStep::CommandWithParams(GCTRL, &[Spec::GCTRL_PARAM]),
        // VCOM Setting
        InitStep::CommandWithParams(VCOMS, &[Spec::VCOMS_PARAM]),
        // LCM Control
        InitStep::CommandWithParams(LCMCTRL, &[Spec::LCMCTRL_PARAM]),
        // VDV and VRH Command Enable
        InitStep::CommandWithParams(VDVVRHEN, &[0x01, 0xFF]),
        // VRH Set
        InitStep::CommandWithParams(VRHS, &[Spec::VRHS_PARAM]),
        // VDV Set
        InitStep::CommandWithParams(VDVS, &[Spec::VDVS_PARAM]),
        // Frame Rate Control
        InitStep::CommandWithParams(FRCTRL2, &[Spec::FRCTRL2_PARAM]),
        // Power Control 1
        InitStep::CommandWithParams(PWCTRL1, &Spec::PWCTRL1_PARAMS),
        // Power Control 2 (Optional)
        InitStep::maybe_cmd_with(PWCTRL2, Spec::PWCTRL2_PARAMS),
        // Equalize time control (Optional)
        InitStep::maybe_cmd_with(EQCTRL, Spec::EQCTRL_PARAMS),
        // Gate Output Selection (Optional)
        InitStep::maybe_cmd_with(GATESEL, Spec::GATESEL_PARAMS),
        // Gamma
        InitStep::CommandWithParams(PVGAMCTRL, &Spec::PVGAMCTRL_PARAMS),
        InitStep::CommandWithParams(NVGAMCTRL, &Spec::NVGAMCTRL_PARAMS),
        // Invert Mode
        InitStep::select_cmd(
            Spec::INVERTED,
            mipidcs::ENTER_INVERT_MODE,
            mipidcs::EXIT_INVERT_MODE,
        ),
        InitStep::CommandWithParams(
            SET_ADDRESS_MODE,
            &[if Spec::BGR {
                AddressMode::BGR.bits()
            } else {
                0u8
            }],
        ),
        // Display On
        InitStep::SingleCommand(mipidcs::SET_DISPLAY_ON),
        InitStep::DelayMs(120),
    ];
}

impl<Spec, RST, B> Panel<B> for St7789<Spec, RST, B>
where
    Spec: St7789Spec,
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

        // Initialize address mode state
        self.inner.address_mode.set(AddressMode::BGR, Spec::BGR);

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
