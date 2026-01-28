#![allow(non_camel_case_types)]
use super::*;

// --- Common Constants for INITR Scenarios ---
// From Adafruit_ST7735.cpp Rcmd1, Rcmd3, and default behaviors.

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
        impl St7735Spec for $type {
            const FRMCTR1_PARAMS: [u8; 3] = INITR_FRMCTR1;
            const FRMCTR2_PARAMS: [u8; 3] = INITR_FRMCTR2;
            const FRMCTR3_PARAMS: [u8; 6] = INITR_FRMCTR3;
            const INVCTR_PARAM: u8 = INITR_INVCTR;
            const PWCTR1_PARAMS: [u8; 3] = INITR_PWCTR1;
            const PWCTR2_PARAM: u8 = INITR_PWCTR2;
            const PWCTR3_PARAMS: [u8; 2] = INITR_PWCTR3;
            const PWCTR4_PARAMS: [u8; 2] = INITR_PWCTR4;
            const PWCTR5_PARAMS: [u8; 2] = INITR_PWCTR5;
            const VMCTR1_PARAM: u8 = INITR_VMCTR1;
            const GMCTRP1_PARAMS: Option<&'static [u8; 16]> = Some(&INITR_GMCTRP1);
            const GMCTRN1_PARAMS: Option<&'static [u8; 16]> = Some(&INITR_GMCTRN1);
        }
    };
}

/// INITR_GREENTAB
/// 1.8" display, 128x160, Offset 2, 1
pub struct InitR_GreenTab;

impl MipidcsSpec for InitR_GreenTab {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 2;
    const PHYSICAL_Y_OFFSET: u16 = 1;

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl_st7735_initr!(InitR_GreenTab);

/// INITR_REDTAB
/// 1.8" display, 128x160, Offset 0, 0
pub struct InitR_RedTab;

impl MipidcsSpec for InitR_RedTab {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl_st7735_initr!(InitR_RedTab);

/// INITR_BLACKTAB
/// 1.8" display, 128x160, Offset 0, 0. Uses RGB color filter unlike others.
pub struct InitR_BlackTab;

impl MipidcsSpec for InitR_BlackTab {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = false;
    const BGR: bool = false; // Adafruit sets MADCTL 0xC0 (RGB) for BlackTab
}

impl_st7735_initr!(InitR_BlackTab);

/// INITR_144GREENTAB
/// 1.44" display, 128x128, Offset 2, 3 (for Rotation 0) or 2, 1? TODO
/// Adafruit: colstart=2, rowstart=3.
pub struct InitR_144GreenTab;

impl MipidcsSpec for InitR_144GreenTab {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 128;
    const PHYSICAL_X_OFFSET: u16 = 2;
    const PHYSICAL_Y_OFFSET: u16 = 3; // Adafruit says 3 for rotation 0

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl_st7735_initr!(InitR_144GreenTab);

/// INITR_MINI160x80
/// 0.96" display, 80x160, Offset 24, 0.
/// Adafruit treats as "BlackTab" for MADCTL (RGB).
pub struct InitR_Mini160x80;

impl MipidcsSpec for InitR_Mini160x80 {
    const PHYSICAL_WIDTH: u16 = 80;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 24;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = false;
    const BGR: bool = false; // Treat as BlackTab (RGB)
}

impl_st7735_initr!(InitR_Mini160x80);

/// INITR_MINI160x80_PLUGIN
/// 0.96" display with plugin FPC. 80x160, Offset 26, 1.
/// Inverted. BGR (default).
pub struct InitR_Mini160x80Plugin;

impl MipidcsSpec for InitR_Mini160x80Plugin {
    const PHYSICAL_WIDTH: u16 = 80;
    const PHYSICAL_HEIGHT: u16 = 160;
    const PHYSICAL_X_OFFSET: u16 = 26;
    const PHYSICAL_Y_OFFSET: u16 = 1;

    const INVERTED: bool = true;
    const BGR: bool = true;
}

impl_st7735_initr!(InitR_Mini160x80Plugin);
