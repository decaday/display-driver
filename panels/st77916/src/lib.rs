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
        InitStep::CommandWithParams(CSC1, &[0x01]),
        InitStep::CommandWithParams(CSC2, &[0x01]),
        InitStep::CommandWithParams(0xB0, &[0x69]),
        InitStep::CommandWithParams(0xB1, &[0x4A]),
        InitStep::CommandWithParams(0xB2, &[0x2F]),
        InitStep::CommandWithParams(0xB3, &[0x01]),
        InitStep::CommandWithParams(0xB4, &[0x69]),
        InitStep::CommandWithParams(0xB5, &[0x45]),
        InitStep::CommandWithParams(0xB6, &[0xAB]),
        InitStep::CommandWithParams(0xB7, &[0x41]),
        InitStep::CommandWithParams(0xB8, &[0x86]),
        InitStep::CommandWithParams(0xB9, &[0x15]),
        InitStep::CommandWithParams(0xBA, &[0x00]),
        InitStep::CommandWithParams(0xBB, &[0x08]),
        InitStep::CommandWithParams(0xBC, &[0x08]),
        InitStep::CommandWithParams(0xBD, &[0x00]),
        InitStep::CommandWithParams(0xBE, &[0x00]),
        InitStep::CommandWithParams(0xBF, &[0x07]),
        InitStep::CommandWithParams(0xC0, &[0x80]),
        InitStep::CommandWithParams(0xC1, &[0x10]),
        InitStep::CommandWithParams(0xC2, &[0x37]),
        InitStep::CommandWithParams(0xC3, &[0x80]),
        InitStep::CommandWithParams(0xC4, &[0x10]),
        InitStep::CommandWithParams(0xC5, &[0x37]),
        InitStep::CommandWithParams(0xC6, &[0xA9]),
        InitStep::CommandWithParams(0xC7, &[0x41]),
        InitStep::CommandWithParams(0xC8, &[0x01]),
        InitStep::CommandWithParams(0xC9, &[0xA9]),
        InitStep::CommandWithParams(0xCA, &[0x41]),
        InitStep::CommandWithParams(0xCB, &[0x01]),
        InitStep::CommandWithParams(0xCC, &[0x7F]),
        InitStep::CommandWithParams(0xCD, &[0x7F]),
        InitStep::CommandWithParams(0xCE, &[0xFF]),
        InitStep::CommandWithParams(0xD0, &[0x91]),
        InitStep::CommandWithParams(0xD1, &[0x68]),
        InitStep::CommandWithParams(0xD2, &[0x68]),
        InitStep::CommandWithParams(0xF5, &[0x00, 0xA5]),
        InitStep::CommandWithParams(CSC2, &[0x10]),
        InitStep::CommandWithParams(CSC1, &[0x00]),
        InitStep::CommandWithParams(CSC1, &[0x02]),
        InitStep::CommandWithParams(GAMCTRP1, &Spec::GAMCTRP1_PARAMS),
        InitStep::CommandWithParams(GAMCTRN1, &Spec::GAMCTRN1_PARAMS),
        InitStep::CommandWithParams(CSC1, &[0x10]),
        InitStep::CommandWithParams(CSC4, &[0x10]),
        InitStep::CommandWithParams(GAMCTRP1, &[0x08]),
        InitStep::CommandWithParams(GAMCTRN1, &[0x00]),
        InitStep::CommandWithParams(0xE2, &[0x00]),
        InitStep::CommandWithParams(0xE3, &[0x00]),
        InitStep::CommandWithParams(0xE4, &[0xE0]),
        InitStep::CommandWithParams(0xE5, &[0x06]),
        InitStep::CommandWithParams(0xE6, &[0x21]),
        InitStep::CommandWithParams(0xE7, &[0x03]),
        InitStep::CommandWithParams(0xE8, &[0x05]),
        InitStep::CommandWithParams(0xE9, &[0x02]),
        InitStep::CommandWithParams(0xEA, &[0xE9]),
        InitStep::CommandWithParams(0xEB, &[0x00]),
        InitStep::CommandWithParams(0xEC, &[0x00]),
        InitStep::CommandWithParams(0xED, &[0x14]),
        InitStep::CommandWithParams(0xEE, &[0xFF]),
        InitStep::CommandWithParams(0xEF, &[0x00]),
        InitStep::CommandWithParams(0xF8, &[0xFF]),
        InitStep::CommandWithParams(0xF9, &[0x00]),
        InitStep::CommandWithParams(0xFA, &[0x00]),
        InitStep::CommandWithParams(0xFB, &[0x30]),
        InitStep::CommandWithParams(0xFC, &[0x00]),
        InitStep::CommandWithParams(0xFD, &[0x00]),
        InitStep::CommandWithParams(0xFE, &[0x00]),
        InitStep::CommandWithParams(0xFF, &[0x00]),
        InitStep::CommandWithParams(0x60, &[0x40]),
        InitStep::CommandWithParams(0x61, &[0x05]),
        InitStep::CommandWithParams(0x62, &[0x00]),
        InitStep::CommandWithParams(0x63, &[0x42]),
        InitStep::CommandWithParams(0x64, &[0xDA]),
        InitStep::CommandWithParams(0x65, &[0x00]),
        InitStep::CommandWithParams(0x66, &[0x00]),
        InitStep::CommandWithParams(0x67, &[0x00]),
        InitStep::CommandWithParams(0x68, &[0x00]),
        InitStep::CommandWithParams(0x69, &[0x00]),
        InitStep::CommandWithParams(0x6A, &[0x00]),
        InitStep::CommandWithParams(0x6B, &[0x00]),
        InitStep::CommandWithParams(0x70, &[0x40]),
        InitStep::CommandWithParams(0x71, &[0x04]),
        InitStep::CommandWithParams(0x72, &[0x00]),
        InitStep::CommandWithParams(0x73, &[0x42]),
        InitStep::CommandWithParams(0x74, &[0xD9]),
        InitStep::CommandWithParams(0x75, &[0x00]),
        InitStep::CommandWithParams(0x76, &[0x00]),
        InitStep::CommandWithParams(0x77, &[0x00]),
        InitStep::CommandWithParams(0x78, &[0x00]),
        InitStep::CommandWithParams(0x79, &[0x00]),
        InitStep::CommandWithParams(0x7A, &[0x00]),
        InitStep::CommandWithParams(0x7B, &[0x00]),
        InitStep::CommandWithParams(0x80, &[0x48]),
        InitStep::CommandWithParams(0x81, &[0x00]),
        InitStep::CommandWithParams(0x82, &[0x07]),
        InitStep::CommandWithParams(0x83, &[0x02]),
        InitStep::CommandWithParams(0x84, &[0xD7]),
        InitStep::CommandWithParams(0x85, &[0x04]),
        InitStep::CommandWithParams(0x86, &[0x00]),
        InitStep::CommandWithParams(0x87, &[0x00]),
        InitStep::CommandWithParams(0x88, &[0x48]),
        InitStep::CommandWithParams(0x89, &[0x00]),
        InitStep::CommandWithParams(0x8A, &[0x09]),
        InitStep::CommandWithParams(0x8B, &[0x02]),
        InitStep::CommandWithParams(0x8C, &[0xD9]),
        InitStep::CommandWithParams(0x8D, &[0x04]),
        InitStep::CommandWithParams(0x8E, &[0x00]),
        InitStep::CommandWithParams(0x8F, &[0x00]),
        InitStep::CommandWithParams(0x90, &[0x48]),
        InitStep::CommandWithParams(0x91, &[0x00]),
        InitStep::CommandWithParams(0x92, &[0x0B]),
        InitStep::CommandWithParams(0x93, &[0x02]),
        InitStep::CommandWithParams(0x94, &[0xDB]),
        InitStep::CommandWithParams(0x95, &[0x04]),
        InitStep::CommandWithParams(0x96, &[0x00]),
        InitStep::CommandWithParams(0x97, &[0x00]),
        InitStep::CommandWithParams(0x98, &[0x48]),
        InitStep::CommandWithParams(0x99, &[0x00]),
        InitStep::CommandWithParams(0x9A, &[0x0D]),
        InitStep::CommandWithParams(0x9B, &[0x02]),
        InitStep::CommandWithParams(0x9C, &[0xDD]),
        InitStep::CommandWithParams(0x9D, &[0x04]),
        InitStep::CommandWithParams(0x9E, &[0x00]),
        InitStep::CommandWithParams(0x9F, &[0x00]),
        InitStep::CommandWithParams(0xA0, &[0x48]),
        InitStep::CommandWithParams(0xA1, &[0x00]),
        InitStep::CommandWithParams(0xA2, &[0x06]),
        InitStep::CommandWithParams(0xA3, &[0x02]),
        InitStep::CommandWithParams(0xA4, &[0xD6]),
        InitStep::CommandWithParams(0xA5, &[0x04]),
        InitStep::CommandWithParams(0xA6, &[0x00]),
        InitStep::CommandWithParams(0xA7, &[0x00]),
        InitStep::CommandWithParams(0xA8, &[0x48]),
        InitStep::CommandWithParams(0xA9, &[0x00]),
        InitStep::CommandWithParams(0xAA, &[0x08]),
        InitStep::CommandWithParams(0xAB, &[0x02]),
        InitStep::CommandWithParams(0xAC, &[0xD8]),
        InitStep::CommandWithParams(0xAD, &[0x04]),
        InitStep::CommandWithParams(0xAE, &[0x00]),
        InitStep::CommandWithParams(0xAF, &[0x00]),
        InitStep::CommandWithParams(0xB0, &[0x48]),
        InitStep::CommandWithParams(0xB1, &[0x00]),
        InitStep::CommandWithParams(0xB2, &[0x0A]),
        InitStep::CommandWithParams(0xB3, &[0x02]),
        InitStep::CommandWithParams(0xB4, &[0xDA]),
        InitStep::CommandWithParams(0xB5, &[0x04]),
        InitStep::CommandWithParams(0xB6, &[0x00]),
        InitStep::CommandWithParams(0xB7, &[0x00]),
        InitStep::CommandWithParams(0xB8, &[0x48]),
        InitStep::CommandWithParams(0xB9, &[0x00]),
        InitStep::CommandWithParams(0xBA, &[0x0C]),
        InitStep::CommandWithParams(0xBB, &[0x02]),
        InitStep::CommandWithParams(0xBC, &[0xDC]),
        InitStep::CommandWithParams(0xBD, &[0x04]),
        InitStep::CommandWithParams(0xBE, &[0x00]),
        InitStep::CommandWithParams(0xBF, &[0x00]),
        InitStep::CommandWithParams(0xC0, &[0x10]),
        InitStep::CommandWithParams(0xC1, &[0x47]),
        InitStep::CommandWithParams(0xC2, &[0x56]),
        InitStep::CommandWithParams(0xC3, &[0x65]),
        InitStep::CommandWithParams(0xC4, &[0x74]),
        InitStep::CommandWithParams(0xC5, &[0x88]),
        InitStep::CommandWithParams(0xC6, &[0x99]),
        InitStep::CommandWithParams(0xC7, &[0x01]),
        InitStep::CommandWithParams(0xC8, &[0xBB]),
        InitStep::CommandWithParams(0xC9, &[0xAA]),
        InitStep::CommandWithParams(0xD0, &[0x10]),
        InitStep::CommandWithParams(0xD1, &[0x47]),
        InitStep::CommandWithParams(0xD2, &[0x56]),
        InitStep::CommandWithParams(0xD3, &[0x65]),
        InitStep::CommandWithParams(0xD4, &[0x74]),
        InitStep::CommandWithParams(0xD5, &[0x88]),
        InitStep::CommandWithParams(0xD6, &[0x99]),
        InitStep::CommandWithParams(0xD7, &[0x01]),
        InitStep::CommandWithParams(0xD8, &[0xBB]),
        InitStep::CommandWithParams(0xD9, &[0xAA]),
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
