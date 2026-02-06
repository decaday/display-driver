#![allow(non_camel_case_types)]
use super::*;

/// 1.54 inch 240x240, offset = (0, 0)
/// Generic Model: Generic240_240_Type1
pub struct TB154;

impl PanelSpec for TB154 {
    const PHYSICAL_WIDTH: u16 = 240;
    const PHYSICAL_HEIGHT: u16 = 240;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const PHYSICAL_X_OFFSET_ROTATED: u16 = 0;
    const PHYSICAL_Y_OFFSET_ROTATED: u16 = 80;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl St7789Spec for TB154 {
    const PORCTRL_PARAMS: [u8; 5] = [0x0C, 0x0C, 0x00, 0x33, 0x33];
    const GCTRL_PARAM: u8 = 0x35;
    const VCOMS_PARAM: u8 = 0x19;
    const LCMCTRL_PARAM: u8 = 0x2C;
    const VRHS_PARAM: u8 = 0x12;
    const VDVS_PARAM: u8 = 0x20;
    const FRCTRL2_PARAM: u8 = 0x0F;
    const PWCTRL1_PARAMS: [u8; 2] = [0xA4, 0xA1];
    const PVGAMCTRL_PARAMS: [u8; 14] = [
        0xD0, 0x04, 0x0D, 0x11, 0x13, 0x2B, 0x3F, 0x54, 0x4C, 0x18, 0x0D, 0x0B, 0x1F, 0x23,
    ];
    const NVGAMCTRL_PARAMS: [u8; 14] = [
        0xD0, 0x04, 0x0C, 0x11, 0x13, 0x2C, 0x3F, 0x44, 0x51, 0x2F, 0x1F, 0x1F, 0x20, 0x23,
    ];
}

/// 1.14 inch 135x240
/// Generic Model: Generic135_240_Type1
pub struct GMT114_02;

impl PanelSpec for GMT114_02 {
    const PHYSICAL_WIDTH: u16 = 135;
    const PHYSICAL_HEIGHT: u16 = 240;
    const PHYSICAL_X_OFFSET: u16 = 52;
    const PHYSICAL_Y_OFFSET: u16 = 40;

    const PHYSICAL_X_OFFSET_ROTATED: u16 = 53;
    const PHYSICAL_Y_OFFSET_ROTATED: u16 = 40;
    const INVERT_TRANSPOSED_OFFSET: bool = true;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl St7789Spec for GMT114_02 {
    const PORCTRL_PARAMS: [u8; 5] = [0x05, 0x05, 0x00, 0x33, 0x33];
    const GCTRL_PARAM: u8 = 0x05;
    const VCOMS_PARAM: u8 = 0x3F;
    const LCMCTRL_PARAM: u8 = 0x2C;
    const VRHS_PARAM: u8 = 0x0F;
    const VDVS_PARAM: u8 = 0x20;
    const FRCTRL2_PARAM: u8 = 0x01;
    const PWCTRL1_PARAMS: [u8; 2] = [0xA4, 0xA1];
    const PVGAMCTRL_PARAMS: [u8; 14] = [
        0xD0, 0x05, 0x09, 0x09, 0x08, 0x14, 0x28, 0x33, 0x3F, 0x07, 0x13, 0x14, 0x28, 0x30,
    ];
    const NVGAMCTRL_PARAMS: [u8; 14] = [
        0xD0, 0x05, 0x09, 0x09, 0x08, 0x03, 0x24, 0x32, 0x32, 0x3B, 0x14, 0x13, 0x28, 0x2F,
    ];
    const PWCTRL2_PARAMS: Option<&'static [u8; 1]> = Some(&[0x03]);
    const EQCTRL_PARAMS: Option<&'static [u8; 3]> = Some(&[0x09, 0x09, 0x08]);
}

/// 1.14 inch 135x240
/// Generic Model: Generic135_240_Type1
pub struct N114_2413THBIG01_H13;

impl PanelSpec for N114_2413THBIG01_H13 {
    const PHYSICAL_WIDTH: u16 = 135;
    const PHYSICAL_HEIGHT: u16 = 240;
    const PHYSICAL_X_OFFSET: u16 = 52;
    const PHYSICAL_Y_OFFSET: u16 = 40;

    const PHYSICAL_X_OFFSET_ROTATED: u16 = 53;
    const PHYSICAL_Y_OFFSET_ROTATED: u16 = 40;
    const INVERT_TRANSPOSED_OFFSET: bool = true;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl St7789Spec for N114_2413THBIG01_H13 {
    const PORCTRL_PARAMS: [u8; 5] = [0x05, 0x05, 0x00, 0x33, 0x33];
    const GCTRL_PARAM: u8 = 0x23;
    const VCOMS_PARAM: u8 = 0x22;
    const LCMCTRL_PARAM: u8 = 0x2C;
    const VRHS_PARAM: u8 = 0x13;
    const VDVS_PARAM: u8 = 0x20;
    const FRCTRL2_PARAM: u8 = 0x0F;
    const PWCTRL1_PARAMS: [u8; 2] = [0xA4, 0xA1];
    const GATESEL_PARAMS: Option<&'static [u8; 1]> = Some(&[0xA1]);
    const PVGAMCTRL_PARAMS: [u8; 14] = [
        0x70, 0x06, 0x0C, 0x08, 0x09, 0x27, 0x2E, 0x34, 0x46, 0x37, 0x13, 0x13, 0x25, 0x2A,
    ];
    const NVGAMCTRL_PARAMS: [u8; 14] = [
        0x70, 0x04, 0x08, 0x09, 0x07, 0x03, 0x2C, 0x42, 0x42, 0x38, 0x14, 0x14, 0x27, 0x2C,
    ];
}
