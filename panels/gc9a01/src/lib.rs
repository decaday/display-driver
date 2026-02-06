#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use display_driver::bus::DisplayBus;
use display_driver::panel::initseq::{sequenced_init, InitStep};
use display_driver::panel::reset::{LCDResetHandler, LCDResetOption};
use display_driver::panel::{Orientation, Panel, PanelSetBrightness};

use display_driver::{ColorFormat, DisplayError};

use display_driver_mipidcs as mipidcs;
use display_driver_mipidcs::{dcs_types::AddressMode, GenericMipidcs};

pub mod consts;
pub mod spec;

use consts::*;
pub use spec::Gc9a01Spec;

/// Driver for the GC9A01 display controller.
pub struct Gc9a01<Spec, RST, B>
where
    Spec: Gc9a01Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    /// Inner generic driver for standard functionality.
    inner: GenericMipidcs<B, Spec, RST>,
}

impl<Spec, RST, B> Gc9a01<Spec, RST, B>
where
    Spec: Gc9a01Spec,
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

    /// Initialization sequence for GC9A01.
    const INIT_STEPS: &'static [InitStep<'static>] = &[
        InitStep::CommandWithParams(INTER_REGISTER_ENABLE_2, &[]), // 0xEF - Use Command 2
        InitStep::CommandWithParams(0xEB, &[0x14]),
        InitStep::CommandWithParams(INTER_REGISTER_ENABLE_1, &[]), // 0xFE - Use Command 1
        InitStep::CommandWithParams(INTER_REGISTER_ENABLE_2, &[]), // 0xEF - Use Command 2, enable inter register
        // There are so many magic here!!!
        InitStep::CommandWithParams(0xEB, &[0x14]),
        InitStep::CommandWithParams(0x84, &[0x40]),
        InitStep::CommandWithParams(0x85, &[0xFF]),
        InitStep::CommandWithParams(0x86, &[0xFF]),
        InitStep::CommandWithParams(0x87, &[0xFF]),
        InitStep::CommandWithParams(0x88, &[0x0A]),
        InitStep::CommandWithParams(0x89, &[0x21]),
        InitStep::CommandWithParams(0x8A, &[0x00]),
        InitStep::CommandWithParams(0x8B, &[0x80]),
        InitStep::CommandWithParams(0x8C, &[0x01]),
        InitStep::CommandWithParams(0x8D, &[0x01]),
        InitStep::CommandWithParams(0x8E, &[0xFF]),
        InitStep::CommandWithParams(0x8F, &[0xFF]),
        // Display Function Control
        InitStep::CommandWithParams(DISPLAY_FUNCTION_CONTROL, &[0x00, 0x20]), // Scan direction S1-S360 G1-32
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
        // Pixel Format Set
        InitStep::CommandWithParams(mipidcs::SET_PIXEL_FORMAT, &[0x05]), // 16bit MCU
        InitStep::CommandWithParams(0x90, &[0x08, 0x08, 0x08, 0x08]),
        InitStep::CommandWithParams(0xBD, &[0x06]),
        InitStep::CommandWithParams(0xBC, &[0x00]),
        InitStep::CommandWithParams(0xFF, &[0x60, 0x01, 0x04]),
        // Power Control
        InitStep::CommandWithParams(POWER_CONTROL_2, &[0x13]), // Vreg1a voltage
        InitStep::CommandWithParams(POWER_CONTROL_3, &[0x13]), // Vreg1b voltage
        InitStep::CommandWithParams(POWER_CONTROL_4, &[0x22]), // Vreg2a voltage
        InitStep::CommandWithParams(0xBE, &[0x11]),
        InitStep::CommandWithParams(0xE1, &[0x10, 0x0E]),
        InitStep::CommandWithParams(0xDF, &[0x21, 0x0C, 0x02]),
        // Gamma
        InitStep::CommandWithParams(SET_GAMMA_1, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A]),
        InitStep::CommandWithParams(SET_GAMMA_2, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6F]),
        InitStep::CommandWithParams(SET_GAMMA_3, &[0x45, 0x09, 0x08, 0x08, 0x26, 0x2A]),
        InitStep::CommandWithParams(SET_GAMMA_4, &[0x43, 0x70, 0x72, 0x36, 0x37, 0x6F]),
        InitStep::CommandWithParams(0xED, &[0x1B, 0x0B]),
        InitStep::CommandWithParams(0xAE, &[0x77]),
        InitStep::CommandWithParams(0xCD, &[0x63]),
        // TODO: Adafruit says remove this line may solve some problems
        // InitStep::CommandWithParams(0x70, &[0x07, 0x07, 0x04, 0x0E, 0x0F, 0x09, 0x07, 0x08, 0x03]),

        // Frame Rate
        InitStep::CommandWithParams(FRAME_RATE_CONTROL, &[0x34]),
        InitStep::CommandWithParams(
            0x62,
            &[
                0x18, 0x0D, 0x71, 0xED, 0x70, 0x70, 0x18, 0x0F, 0x71, 0xEF, 0x70, 0x70,
            ],
        ),
        InitStep::CommandWithParams(
            0x63,
            &[
                0x18, 0x11, 0x71, 0xF1, 0x70, 0x70, 0x18, 0x13, 0x71, 0xF3, 0x70, 0x70,
            ],
        ),
        InitStep::CommandWithParams(0x64, &[0x28, 0x29, 0xF1, 0x01, 0xF1, 0x00, 0x07]),
        InitStep::CommandWithParams(
            0x66,
            &[0x3C, 0x00, 0xCD, 0x67, 0x45, 0x45, 0x10, 0x00, 0x00, 0x00],
        ),
        InitStep::CommandWithParams(
            0x67,
            &[0x00, 0x3C, 0x00, 0x00, 0x00, 0x01, 0x54, 0x10, 0x32, 0x98],
        ),
        InitStep::CommandWithParams(0x74, &[0x10, 0x85, 0x80, 0x00, 0x00, 0x4E, 0x00]),
        InitStep::CommandWithParams(0x98, &[0x3E, 0x07]),
        InitStep::SingleCommand(mipidcs::SET_TEAR_OFF),
        InitStep::SingleCommand(mipidcs::ENTER_INVERT_MODE), // Display Inversion ON
        InitStep::SingleCommand(mipidcs::EXIT_SLEEP_MODE),   // Sleep Out
        // InitStep::SingleCommand(mipidcs::EXIT_IDLE_MODE), // Idle Mode OFF
        InitStep::DelayMs(120),
        InitStep::SingleCommand(mipidcs::SET_DISPLAY_ON),
        InitStep::DelayMs(20),
    ];
}

impl<Spec, RST, B> Panel<B> for Gc9a01<Spec, RST, B>
where
    Spec: Gc9a01Spec,
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

        // Initialize address mode cache
        self.inner.address_mode.set(AddressMode::BGR, Spec::BGR);

        // Execute Initialization Sequence
        // `copied()` only copies the items during iteration; it does not copy the entire sequence
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

impl<Spec, RST, B> PanelSetBrightness<B> for Gc9a01<Spec, RST, B>
where
    Spec: Gc9a01Spec,
    RST: OutputPin,
    B: DisplayBus,
{
    async fn set_brightness(
        &mut self,
        bus: &mut B,
        brightness: u8,
    ) -> Result<(), DisplayError<B::Error>> {
        bus.write_cmd_with_params(&[WRITE_DISPLAY_BRIGHTNESS], &[brightness])
            .await
            .map_err(DisplayError::BusError)
    }
}
