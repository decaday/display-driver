#![allow(non_camel_case_types)]
use super::*;

// Common Generic Constants (Based on LovyanGFX/TFT_eSPI)
// These define the generic initialization many panels use.

pub const GENERIC_PORCTRL_PARAMS: [u8; 5] = [0x0C, 0x0C, 0x00, 0x33, 0x33];
pub const GENERIC_GCTRL_PARAM: u8 = 0x35;
pub const GENERIC_VCOMS_PARAM: u8 = 0x28;
pub const GENERIC_LCMCTRL_PARAM: u8 = 0x0C;
pub const GENERIC_VRHS_PARAM: u8 = 0x10;
pub const GENERIC_VDVS_PARAM: u8 = 0x20;
pub const GENERIC_FRCTRL2_PARAM: u8 = 0x0F;
pub const GENERIC_PWCTRL1_PARAMS: [u8; 2] = [0xA4, 0xA1];
pub const GENERIC_PVGAMCTRL_PARAMS: [u8; 14] = [
    0xD0, 0x00, 0x02, 0x07, 0x0A, 0x28, 0x32, 0x44, 0x42, 0x06, 0x0E, 0x12, 0x14, 0x17,
];
pub const GENERIC_NVGAMCTRL_PARAMS: [u8; 14] = [
    0xD0, 0x00, 0x02, 0x07, 0x0A, 0x28, 0x31, 0x54, 0x47, 0x0E, 0x1C, 0x17, 0x1B, 0x1E,
];

// Macro to implement St7789Spec for types using these generic constants
#[macro_export]
macro_rules! impl_st7789_generic {
    ($type:ty) => {
        impl crate::St7789Spec for $type {
            const PORCTRL_PARAMS: [u8; 5] = crate::spec::generic::GENERIC_PORCTRL_PARAMS;
            const GCTRL_PARAM: u8 = crate::spec::generic::GENERIC_GCTRL_PARAM;
            const VCOMS_PARAM: u8 = crate::spec::generic::GENERIC_VCOMS_PARAM;
            const LCMCTRL_PARAM: u8 = crate::spec::generic::GENERIC_LCMCTRL_PARAM;
            const VRHS_PARAM: u8 = crate::spec::generic::GENERIC_VRHS_PARAM;
            const VDVS_PARAM: u8 = crate::spec::generic::GENERIC_VDVS_PARAM;
            const FRCTRL2_PARAM: u8 = crate::spec::generic::GENERIC_FRCTRL2_PARAM;
            const PWCTRL1_PARAMS: [u8; 2] = crate::spec::generic::GENERIC_PWCTRL1_PARAMS;
            const PVGAMCTRL_PARAMS: [u8; 14] = crate::spec::generic::GENERIC_PVGAMCTRL_PARAMS;
            const NVGAMCTRL_PARAMS: [u8; 14] = crate::spec::generic::GENERIC_NVGAMCTRL_PARAMS;
        }
    };
}

/// Generic ST7789 spec for 240x320 displays
/// Uses the Generic initialization sequence.
/// 240x320, offset = (0, 0), Inverted = true, BGR = false (RGB)
pub struct Generic240x320;

impl MipidcsSpec for Generic240x320 {
    const PHYSICAL_WIDTH: u16 = 240;
    const PHYSICAL_HEIGHT: u16 = 320;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl_st7789_generic!(Generic240x320);

/// Generic ST7789 spec for 240x240 square displays
/// Uses the Generic initialization sequence.
/// 240x240, offset = (0, 0), Inverted = true, BGR = false
pub struct Generic240x240;

impl MipidcsSpec for Generic240x240 {
    const PHYSICAL_WIDTH: u16 = 240;
    const PHYSICAL_HEIGHT: u16 = 240;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl_st7789_generic!(Generic240x240);
