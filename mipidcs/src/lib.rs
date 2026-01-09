#![no_std]

pub mod consts;
pub mod dcs_types;

use display_driver::DisplayBus;
use core::marker::PhantomData;

pub struct GenericMipidcs<DI: DisplayBus> {
    _phantom: PhantomData<DI>
}

use crate::consts::*;
use crate::dcs_types::*;

impl<DI: DisplayBus> GenericMipidcs<DI> {
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
    pub async fn set_column_address(&self, bus: &mut DI, start: u16, end: u16) -> Result<(), DI::Error> {
        let params = AddressRange::new(start, end);
        bus.write_cmd_with_params(&[SET_COLUMN_ADDRESS], &params.0).await
    }

    /// Set the page (row) address window (Command 0x2B).
    pub async fn set_page_address(&self, bus: &mut DI, start: u16, end: u16) -> Result<(), DI::Error> {
        let params = AddressRange::new(start, end);
        bus.write_cmd_with_params(&[SET_PAGE_ADDRESS], &params.0).await
    }

    /// Set the Address Mode (Memory Data Access Control, aka. MADCTL - Command 0x36).
    pub async fn set_address_mode(&self, bus: &mut DI, mode: AddressMode) -> Result<(), DI::Error> {
        bus.write_cmd_with_params(&[SET_ADDRESS_MODE], &[mode.bits()]).await
    }

    /// Set the Pixel Format (Command 0x3A).
    pub async fn set_pixel_format(&self, bus: &mut DI, mode: PixelFormat) -> Result<(), DI::Error> {
        bus.write_cmd_with_params(&[SET_PIXEL_FORMAT], &[mode.0]).await
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
}