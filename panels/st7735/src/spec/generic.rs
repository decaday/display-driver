#![allow(non_camel_case_types)]
use super::*;

// Common Constants for INITR Scenarios From Adafruit_ST7735.cpp

// Frame Rate Control
pub const INITR_FRMCTR1: [u8; 3] = [0x01, 0x2C, 0x2D];
pub const INITR_FRMCTR2: [u8; 3] = [0x01, 0x2C, 0x2D];
pub const INITR_FRMCTR3: [u8; 6] = [0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D];

// Inversion Control (0x07 = No inversion)
pub const INITR_INVCTR: u8 = 0x07;

// Power Control
pub const INITR_PWCTR1: [u8; 3] = [0xA2, 0x02, 0x84];
pub const INITR_PWCTR2: u8 = 0xC5;
pub const INITR_PWCTR3: [u8; 2] = [0x0A, 0x00];
pub const INITR_PWCTR4: [u8; 2] = [0x8A, 0x2A];
pub const INITR_PWCTR5: [u8; 2] = [0x8A, 0xEE];

// VCOM Control
pub const INITR_VMCTR1: u8 = 0x0E;

// Gamma Sequences (Magical Unicorn Dust and Sparkles)
pub const INITR_GMCTRP1: [u8; 16] = [
    0x02, 0x1C, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2D, 0x29, 0x25, 0x2B, 0x39, 0x00, 0x01, 0x03, 0x10,
];
pub const INITR_GMCTRN1: [u8; 16] = [
    0x03, 0x1D, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D, 0x2E, 0x2E, 0x37, 0x3F, 0x00, 0x00, 0x02, 0x10,
];

// --- Implementations ---
#[macro_export]
macro_rules! impl_st7735_initr {
    ($type:ty) => {
        impl crate::St7735Spec for $type {
            const FRMCTR1_PARAMS: [u8; 3] = crate::spec::generic::INITR_FRMCTR1;
            const FRMCTR2_PARAMS: [u8; 3] = crate::spec::generic::INITR_FRMCTR2;
            const FRMCTR3_PARAMS: [u8; 6] = crate::spec::generic::INITR_FRMCTR3;
            const INVCTR_PARAM: u8 = crate::spec::generic::INITR_INVCTR;
            const PWCTR1_PARAMS: [u8; 3] = crate::spec::generic::INITR_PWCTR1;
            const PWCTR2_PARAM: u8 = crate::spec::generic::INITR_PWCTR2;
            const PWCTR3_PARAMS: [u8; 2] = crate::spec::generic::INITR_PWCTR3;
            const PWCTR4_PARAMS: [u8; 2] = crate::spec::generic::INITR_PWCTR4;
            const PWCTR5_PARAMS: [u8; 2] = crate::spec::generic::INITR_PWCTR5;
            const VMCTR1_PARAM: u8 = crate::spec::generic::INITR_VMCTR1;
            const GMCTRP1_PARAMS: Option<&'static [u8; 16]> =
                Some(&crate::spec::generic::INITR_GMCTRP1);
            const GMCTRN1_PARAMS: Option<&'static [u8; 16]> =
                Some(&crate::spec::generic::INITR_GMCTRN1);
        }
    };
}

/// 128x160, offset = (0, 0), Inverted = false, BGR = true
/// aka. RedTab(Adafruit and TFT_eSPI)
pub struct Generic128_160Type1;
impl PanelSpec for Generic128_160Type1 {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl_st7735_initr!(Generic128_160Type1);

/// 128x160, offset = (0, 0), Inverted = false, BGR = false
/// aka. BLACKTAB(Adafruit and TFT_eSPI)
pub struct Generic128_160Type2;
impl PanelSpec for Generic128_160Type2 {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = false;
    const BGR: bool = false;
}

impl_st7735_initr!(Generic128_160Type2);

/// 128x160, offset = (2, 1), Inverted = false, BGR = true
/// aka. GREENTAB(Adafruit and TFT_eSPI)
pub struct Generic128_160Type3;
impl PanelSpec for Generic128_160Type3 {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 2;
    const PHYSICAL_Y_OFFSET: u16 = 1;

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl_st7735_initr!(Generic128_160Type3);

/// 128x160, offset = (2, 1), Inverted = false, BGR = false
/// aka. GREENTAB2(TFT_eSPI) or BOE Panel
pub struct Generic128_160Type4;
impl PanelSpec for Generic128_160Type4 {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 2;
    const PHYSICAL_Y_OFFSET: u16 = 1;

    const INVERTED: bool = false;
    const BGR: bool = false;
}

impl_st7735_initr!(Generic128_160Type4);

/// 128x160, offset = (2, 1) and (2, 3) (after rotated), Inverted = false, BGR = true
/// aka. GREENTAB3(TFT_eSPI)
pub struct Generic128_160Type5;
impl PanelSpec for Generic128_160Type5 {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 2;
    const PHYSICAL_Y_OFFSET: u16 = 1;

    const PHYSICAL_X_OFFSET_ROTATED: u16 = 2;
    const PHYSICAL_Y_OFFSET_ROTATED: u16 = 3;

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl_st7735_initr!(Generic128_160Type5);

/// 80x160, offset = (24, 0), Inverted = false, BGR = false
/// aka. MINI160x80(Adafruit), BOE Panel
pub struct Generic80_160_Type1;
impl PanelSpec for Generic80_160_Type1 {
    const PHYSICAL_WIDTH: u16 = 80;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 24;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = false;
    const BGR: bool = false;
}

impl_st7735_initr!(Generic80_160_Type1);

/// 80x160, offset = (24, 0), Inverted = false, BGR = true
/// aka. REDTAB160x80(TFT_eSPI)
pub struct Generic80_160_Type2;
impl PanelSpec for Generic80_160_Type2 {
    const PHYSICAL_WIDTH: u16 = 80;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 24;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl_st7735_initr!(Generic80_160_Type2);

/// 80x160, offset = (26, 1), Inverted = true, BGR = true
/// aka. MINI160x80PLUGIN(Adafruit), GREENTAB160x80(TFT_eSPI), HannStar Panel
pub struct Generic80_160_Type3;
impl PanelSpec for Generic80_160_Type3 {
    const PHYSICAL_WIDTH: u16 = 80;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 26;
    const PHYSICAL_Y_OFFSET: u16 = 1;

    const INVERTED: bool = true;
    const BGR: bool = true;
}

impl_st7735_initr!(Generic80_160_Type3);

/// 128x128, offset = (2, 1) and (2, 3) (after rotated), Inverted = false, BGR = true
/// aka. 144GREENTAB(Adafruit), HALLOWING(Adafruit)
pub struct Generic128_128_Type1;
impl PanelSpec for Generic128_128_Type1 {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 128;
    const PHYSICAL_X_OFFSET: u16 = 2;
    const PHYSICAL_Y_OFFSET: u16 = 1;

    const PHYSICAL_X_OFFSET_ROTATED: u16 = 2;
    const PHYSICAL_Y_OFFSET_ROTATED: u16 = 3;

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl_st7735_initr!(Generic128_128_Type1);

/// 128x128, offset = (0, 0) and (0, 32) (after rotated), Inverted = false, BGR = true
/// aka. GREENTAB128(TFT_eSPI)
pub struct Generic128_128_Type2;
impl PanelSpec for Generic128_128_Type2 {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 128;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const PHYSICAL_X_OFFSET_ROTATED: u16 = 0;
    const PHYSICAL_Y_OFFSET_ROTATED: u16 = 32;

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl_st7735_initr!(Generic128_128_Type2);
