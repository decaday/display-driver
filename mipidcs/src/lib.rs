#![no_std]

pub mod consts;
pub mod dcs_types;
pub mod spec;

use core::marker::PhantomData;
use display_driver::display_bus::{DisplayBus, Metadata};
use display_driver::panel::{InitStep, LCDResetOption, LCDReseter, Orientation, Panel, addres_window_param_u8, sequenced_init};
use display_driver::{ColorFormat, DisplayError};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use crate::consts::*;
use crate::dcs_types::*;
use crate::spec::DisplaySpec;

/// Generic MIPI DCS display driver.
///
/// This driver uses standard MIPI DCS commands to control the display.
pub struct GenericMipidcs<DI, Spec, RST>
where
    DI: DisplayBus,
    Spec: DisplaySpec,
    RST: OutputPin,
{
    reset_pin: LCDResetOption<RST>,
    /// The current Address Mode (MADCTL) setting.
    pub address_mode: AddressMode,
    _phantom: PhantomData<(DI, Spec)>,
}

impl<DI, Spec, RST> GenericMipidcs<DI, Spec, RST>
where
    DI: DisplayBus,
    Spec: DisplaySpec,
    RST: OutputPin,
{
    /// Creates a new generic MIPI DCS driver.
    pub fn new(reset_pin: LCDResetOption<RST>, address_mode: AddressMode) -> Self {
        Self {
            reset_pin,
            address_mode,
            _phantom: PhantomData,
        }
    }

    /// Software reset on the display controller (Command 0x01).
    pub async fn soft_reset(&self, bus: &mut DI) -> Result<(), DI::Error> {
        bus.write_cmds(&[SOFT_RESET]).await
    }

    /// Enter Sleep Mode (Command 0x10).
    pub async fn enter_sleep_mode(&self, bus: &mut DI) -> Result<(), DI::Error> {
        bus.write_cmds(&[ENTER_SLEEP_MODE]).await
    }

    /// Exit Sleep Mode (Command 0x11).
    pub async fn exit_sleep_mode(&self, bus: &mut DI) -> Result<(), DI::Error> {
        bus.write_cmds(&[EXIT_SLEEP_MODE]).await
    }

    /// Turn the display panel OFF (Command 0x28).
    pub async fn set_display_off(&self, bus: &mut DI) -> Result<(), DI::Error> {
        bus.write_cmds(&[SET_DISPLAY_OFF]).await
    }

    /// Turn the display panel ON (Command 0x29).
    pub async fn set_display_on(&self, bus: &mut DI) -> Result<(), DI::Error> {
        bus.write_cmds(&[SET_DISPLAY_ON]).await
    }

    /// Set the column address window (Command 0x2A).
    pub async fn set_column_address(
        &self,
        bus: &mut DI,
        start: u16,
        end: u16,
    ) -> Result<(), DI::Error> {
        let params = AddressRange::new(start + Spec::COL_OFFSET, end + Spec::COL_OFFSET);
        bus.write_cmd_with_params(&[SET_COLUMN_ADDRESS], &params.0)
            .await
    }

    /// Set the page (row) address window (Command 0x2B).
    pub async fn set_page_address(
        &self,
        bus: &mut DI,
        start: u16,
        end: u16,
    ) -> Result<(), DI::Error> {
        let params = AddressRange::new(start + Spec::ROW_OFFSET, end + Spec::ROW_OFFSET);
        bus.write_cmd_with_params(&[SET_PAGE_ADDRESS], &params.0)
            .await
    }

    /// Set the Address Mode (Memory Data Access Control, aka. MADCTL - Command 0x36).
    pub async fn set_address_mode(&self, bus: &mut DI, mode: AddressMode) -> Result<(), DI::Error> {
        bus.write_cmd_with_params(&[SET_ADDRESS_MODE], &[mode.bits()])
            .await
    }

    /// Set the Pixel Format (Command 0x3A).
    pub async fn set_pixel_format(&self, bus: &mut DI, mode: PixelFormat) -> Result<(), DI::Error> {
        bus.write_cmd_with_params(&[SET_PIXEL_FORMAT], &[mode.0])
            .await
    }

    /// Set Inversion Mode (Command 0x20 / 0x21).
    ///
    /// `true` enters Invert Mode (0x21), `false` exits Invert Mode (0x20).
    pub async fn set_invert_mode(&self, bus: &mut DI, inverted: bool) -> Result<(), DI::Error> {
        match inverted {
            true => bus.write_cmds(&[ENTER_INVERT_MODE]).await,
            false => bus.write_cmds(&[EXIT_INVERT_MODE]).await,
        }
    }

    const INIT_STEPS: [InitStep<'_>; 8] = [
        InitStep::SingleCommand(SOFT_RESET),
        InitStep::DelayMs(120),
        InitStep::SingleCommand(EXIT_SLEEP_MODE),
        InitStep::DelayMs(120),
        InitStep::CommandWithParams((SET_COLUMN_ADDRESS, &addres_window_param_u8(0, Spec::WIDTH, Spec::COL_OFFSET))),
        InitStep::CommandWithParams((SET_PAGE_ADDRESS, &addres_window_param_u8(0, Spec::HEIGHT, Spec::ROW_OFFSET))),
        // Power On
        InitStep::SingleCommand(SET_DISPLAY_ON),
        InitStep::DelayMs(20),
    ];
}

impl<DI, Spec, RST> Panel<DI> for GenericMipidcs<DI, Spec, RST>
where
    DI: DisplayBus,
    Spec: DisplaySpec,
    RST: OutputPin,
{
    async fn init<D: DelayNs>(&mut self, bus: &mut DI, mut delay: D) -> Result<(), DI::Error> {
        // Hardware Reset
        let mut reseter = LCDReseter::new(&mut self.reset_pin, bus, &mut delay, 10);
        reseter.reset().await?;

        sequenced_init(Self::INIT_STEPS.into_iter(), &mut delay, bus).await?;
        self.set_address_mode(bus, self.address_mode).await
    }

    fn size(&self) -> (u16, u16) {
        (Spec::WIDTH, Spec::HEIGHT)
    }

    async fn set_window(
        &mut self,
        bus: &mut DI,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), DI::Error> {
        let x_start = x0 + Spec::COL_OFFSET;
        let x_end = x1 + Spec::COL_OFFSET;
        let y_start = y0 + Spec::ROW_OFFSET;
        let y_end = y1 + Spec::ROW_OFFSET;

        self.set_column_address(bus, x_start, x_end).await?;
        self.set_page_address(bus, y_start, y_end).await
    }

    async fn write_pixels(
        &mut self,
        bus: &mut DI,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        buffer: &[u8],
    ) -> Result<(), DI::Error> {
        self.set_window(bus, x0, y0, x1, y1).await?;

        let metadata = Metadata {
            width: x1 - x0 + 1,
            height: y1 - y0 + 1,
        };

        bus.write_pixels(&[WRITE_MEMORY_START], &[], buffer, metadata)
            .await
            .map_err(|e| match e {
                DisplayError::BusError(be) => be,
                _ => panic!("Unsupported display bus operation during write_pixels"),
            })
    }

async fn set_color_format(
        &mut self,
        bus: &mut DI,
        color_format: ColorFormat,
    ) -> Result<(), DisplayError<DI::Error>> {
        let bits = color_format.size_bits();
        
        // Use from_bit_count as requested
        let pf_type = PixelFormatType::from_bit_count(bits)
            .ok_or(DisplayError::Unsupported)?;
            
        // Use dbi_and_dpi for better compatibility
        let pf = PixelFormat::dbi_and_dpi(pf_type);

        self.set_pixel_format(bus, pf)
            .await
            .map_err(DisplayError::BusError)
    }

    async fn set_orientation(
        &mut self,
        bus: &mut DI,
        orientation: Orientation,
    ) -> Result<(), DisplayError<DI::Error>> {
        let mut mode = self.address_mode;
        
        // Clean up ONLY orientation related bits, preserving others (like BGR, Flip, Latch Order)
        mode.remove(AddressMode::MX | AddressMode::MY | AddressMode::MV);

        // Calculate new orientation bits
        let (mx, my, mv) = match orientation {
            Orientation::Deg0 => (false, false, false),
            Orientation::Deg90 => (true, false, true),
            Orientation::Deg180 => (true, true, false),
            Orientation::Deg270 => (false, true, true),
        };

        mode.set(AddressMode::MX, mx);
        mode.set(AddressMode::MY, my);
        mode.set(AddressMode::MV, mv);

        self.address_mode = mode;
        self.set_address_mode(bus, mode)
            .await
            .map_err(DisplayError::BusError)
    }
}