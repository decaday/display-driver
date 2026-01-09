#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use display_driver::display_bus::{DisplayBus, Metadata};
use display_driver::panel::{InitStep, LCDResetOption, LCDReseter, Orientation, Panel, addres_window_param_u8, sequenced_init};
use display_driver::{ColorFormat, DisplayError};

pub mod consts;
pub mod spec;

use spec::DisplaySpec;
use consts::*;

/// CO5300 LCD Driver.
pub struct Co5300<Spec, RST, B>
where
    Spec: DisplaySpec,
    RST: OutputPin,
    B: DisplayBus,
{
    reset_pin: LCDResetOption<RST>,
    _spec: core::marker::PhantomData<Spec>,
    _bus: core::marker::PhantomData<B>,
}

impl<Spec, RST, B> Co5300<Spec, RST, B>
where
    Spec: DisplaySpec,
    RST: OutputPin,
    B: DisplayBus,
{
    /// Creates a new driver instance.
    pub fn new(reset_pin: LCDResetOption<RST>) -> Self {
        Self {
            reset_pin,
            _spec: core::marker::PhantomData,
            _bus: core::marker::PhantomData,
        }
    }

    const INIT_STEPS: [InitStep<'_>; 18] = [
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
            InitStep::CommandWithParams((COLOR_MODE, &[0x55])), // Default to RGB565
            InitStep::CommandWithParams((TEARING_EFFECT_ON, &[0x00])),
            InitStep::CommandWithParams((WRITE_CTRL_DISPLAY, &[0x20])),
            InitStep::CommandWithParams((WRHBMDISBV, &[0xFF])),
            InitStep::CommandWithParams((CASET, &addres_window_param_u8(0, Spec::WIDTH, Spec::COL_OFFSET))),
            InitStep::CommandWithParams((RASET, &addres_window_param_u8(0, Spec::HEIGHT, Spec::ROW_OFFSET))),
            // Power On
            InitStep::SingleCommand(SLEEP_OUT),
            InitStep::DelayMs(120),
            InitStep::SingleCommand(DISPLAY_ON),
            InitStep::DelayMs(70),
        ];
}

impl<Spec, RST, B> Panel<B> for Co5300<Spec, RST, B>
where
    Spec: DisplaySpec,
    RST: OutputPin,
    B: DisplayBus,
{
    async fn init<D: DelayNs>(&mut self, bus: &mut B, mut delay: D) -> Result<(), B::Error> {
        // Hardware Reset
        let mut reseter = LCDReseter::new(&mut self.reset_pin, bus, &mut delay, 10);
        reseter.reset().await?;

        // Execute Sequence
        sequenced_init(Self::INIT_STEPS.into_iter(), &mut delay, bus).await
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

        bus.write_cmd_with_params(&[CASET], &x_buf).await?;
        bus.write_cmd_with_params(&[RASET], &y_buf).await?;

        Ok(())
    }

    async fn write_pixels(
        &mut self,
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        buffer: &[u8],
    ) -> Result<(), B::Error> {
        self.set_window(bus, x0, y0, x1, y1).await?;

        let metadata = Metadata {
            width: x1 - x0 + 1,
            height: y1 - y0 + 1,
        };

        // Delegate to the bus implementation to handle the transfer
        bus.write_pixels(&[WRITE_RAM], &[], buffer, metadata)
            .await
            .map_err(|e| match e {
                DisplayError::BusError(be) => be,
                _ => panic!("Unsupported display bus operation during write_pixels"),
            })
    }

    async fn set_color_format(
        &mut self,
        bus: &mut B,
        color_format: ColorFormat,
    ) -> Result<(), DisplayError<B::Error>> {
        let param = match color_format {
            ColorFormat::RGB565 => ColorMode::RGB565,
            ColorFormat::RGB666 => ColorMode::RGB666,
            _ => return Err(DisplayError::Unsupported),
        };

        bus.write_cmd_with_params(&[COLOR_MODE], &[param as u8])
            .await
            .map_err(DisplayError::BusError)
    }

    /// Sets the display brightness (0-255).
    async fn set_brightness(&mut self, bus: &mut B, value: u8) -> Result<(), DisplayError<B::Error>> {
        bus.write_cmd_with_params(&[WBRIGHT], &[value]).await.map_err(DisplayError::BusError)
    }

    async fn set_orientation(&mut self, 
            _bus: &mut B,
            orientation: Orientation,
        ) -> Result<(), DisplayError<B::Error>> {
        let _orientation = match orientation {
            Orientation::Deg0 => Some(REG_ORIENTATION_PORTRAIT),
            Orientation::Deg90 => Some(REG_ORIENTATION_LANDSCAPE),
            Orientation::Deg180 => Some(REG_ORIENTATION_LANDSCAPE_ROT180),
            Orientation::Deg270 => None,
        };
        // if let Some(orientation) = orientation {
        //     bus.write_cmd_with_params(&[todo!()], &[orientation])
        //     .await
        //         .map_err(DisplayError::BusError)
        // } else {
        //     Err(DisplayError::Unsupported)
        // }
        Err(DisplayError::Unsupported)
    }
}