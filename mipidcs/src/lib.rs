#![no_std]

pub mod consts;
pub mod dcs_types;
pub mod display_bus;

use core::marker::PhantomData;
use display_driver::bus::DisplayBus;
use display_driver::panel::{initseq::InitStep, reset::LCDResetOption};
use embedded_hal::digital::OutputPin;

pub use crate::consts::*;
pub use crate::dcs_types::*;

/// A generic driver for MIPI DCS compliant displays.
///
/// This struct implements standard MIPI Display Command Set (MIPI DCS) operations such as setting address windows,
/// controlling sleep modes, and handling pixel formats.
/// It is designed to be embedded within specific panel drivers to handle the common DCS functionality.
pub struct GenericMipidcs<B, S, RST>
where
    B: DisplayBus,
    S: DisplaySize,
    RST: OutputPin,
{
    pub reset_pin: LCDResetOption<RST>,
    /// The current Address Mode (MADCTL) setting.
    pub address_mode: AddressMode,
    _phantom: PhantomData<(B, S)>,
}

impl<B, S, RST> GenericMipidcs<B, S, RST>
where
    B: DisplayBus,
    S: DisplaySize,
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
    pub async fn soft_reset(&self, bus: &mut B) -> Result<(), B::Error> {
        bus.write_cmds(&[SOFT_RESET]).await
    }

    /// Enter Sleep Mode (Command 0x10).
    pub async fn enter_sleep_mode(&self, bus: &mut B) -> Result<(), B::Error> {
        bus.write_cmds(&[ENTER_SLEEP_MODE]).await
    }

    /// Exit Sleep Mode (Command 0x11).
    pub async fn exit_sleep_mode(&self, bus: &mut B) -> Result<(), B::Error> {
        bus.write_cmds(&[EXIT_SLEEP_MODE]).await
    }

    /// Turn the display panel OFF (Command 0x28).
    pub async fn set_display_off(&self, bus: &mut B) -> Result<(), B::Error> {
        bus.write_cmds(&[SET_DISPLAY_OFF]).await
    }

    /// Turn the display panel ON (Command 0x29).
    pub async fn set_display_on(&self, bus: &mut B) -> Result<(), B::Error> {
        bus.write_cmds(&[SET_DISPLAY_ON]).await
    }

    /// Set the column address window (Command 0x2A).
    pub async fn set_column_address(
        &self,
        bus: &mut B,
        start: u16,
        end: u16,
    ) -> Result<(), B::Error> {
        let params = AddressRange::new_with_offset(start, end, S::COL_OFFSET);
        bus.write_cmd_with_params(&[SET_COLUMN_ADDRESS], params.as_bytes())
            .await
    }

    /// Set the page (row) address window (Command 0x2B).
    pub async fn set_page_address(
        &self,
        bus: &mut B,
        start: u16,
        end: u16,
    ) -> Result<(), B::Error> {
        let params = AddressRange::new_with_offset(start, end, S::ROW_OFFSET);
        bus.write_cmd_with_params(&[SET_PAGE_ADDRESS], params.as_bytes())
            .await
    }

    pub async fn set_address_window(
        &self,
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), B::Error> {
        bus.write_cmd_with_params(&[SET_COLUMN_ADDRESS], AddressRange::new_with_offset(x0, x1, S::COL_OFFSET).as_bytes())
            .await?;

        bus.write_cmd_with_params(&[SET_PAGE_ADDRESS], AddressRange::new_with_offset(y0, y1, S::ROW_OFFSET).as_bytes())
            .await
    }

    /// Set the Address Mode (Memory Data Access Control, aka. MADCTL - Command 0x36).
    pub async fn set_address_mode(
        &mut self,
        bus: &mut B,
        mode: AddressMode,
    ) -> Result<(), B::Error> {
        self.address_mode = mode;
        bus.write_cmd_with_params(&[SET_ADDRESS_MODE], &[mode.bits()])
            .await
    }

    /// Set the BGR/RGB order in Address Mode (MADCTL).
    pub async fn set_bgr_order(&mut self, bus: &mut B, bgr: bool) -> Result<(), B::Error> {
        self.address_mode.set(AddressMode::BGR, bgr);
        bus.write_cmd_with_params(&[SET_ADDRESS_MODE], &[self.address_mode.bits()])
            .await
    }

    /// Set the Pixel Format (Command 0x3A).
    pub async fn set_pixel_format(&self, bus: &mut B, mode: PixelFormat) -> Result<(), B::Error> {
        bus.write_cmd_with_params(&[SET_PIXEL_FORMAT], &[mode.0])
            .await
    }

    /// Set Inversion Mode (Command 0x20 / 0x21).
    ///
    /// `true` enters Invert Mode (0x21), `false` exits Invert Mode (0x20).
    pub async fn set_invert_mode(&self, bus: &mut B, inverted: bool) -> Result<(), B::Error> {
        match inverted {
            true => bus.write_cmds(&[ENTER_INVERT_MODE]).await,
            false => bus.write_cmds(&[EXIT_INVERT_MODE]).await,
        }
    }

    const INIT_STEPS: [InitStep<'_>; 4] = [
        InitStep::SingleCommand(EXIT_SLEEP_MODE),
        InitStep::DelayMs(120),
        // Power On
        InitStep::SingleCommand(SET_DISPLAY_ON),
        InitStep::DelayMs(20),
    ];
}

/// Display Specification Trait.
pub trait DisplaySize {
    /// Screen width in pixels.
    const WIDTH: u16;
    /// Screen height in pixels.
    const HEIGHT: u16;
    /// Column offset in pixels (default 0).
    const COL_OFFSET: u16 = 0;
    /// Row offset in pixels (default 0).
    const ROW_OFFSET: u16 = 0;
}
