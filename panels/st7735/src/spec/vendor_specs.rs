#![allow(non_camel_case_types)]
use super::*;

/// 0.96 inch TFT IPS 80x160
pub struct XX096T_IF09;

impl MipidcsSpec for XX096T_IF09 {
    const WIDTH: u16 = 80;
    const HEIGHT: u16 = 160;
    const COL_OFFSET: u16 = 26;
    const ROW_OFFSET: u16 = 1;
    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl St7735Spec for XX096T_IF09 {
    // Rate = fosc/(1x2+40) * (LINE+2C+2D)
    const FRMCTR1_PARAMS: [u8; 3] = [0x01, 0x2C, 0x2D];
    const FRMCTR2_PARAMS: [u8; 3] = [0x01, 0x2C, 0x2D];
    const FRMCTR3_PARAMS: [u8; 6] = [0x01, 0x2C, 0x2D, 0x01, 0x2C, 0x2D];

    // No inversion (0x07)
    const INVCTR_PARAM: u8 = 0x07;

    // -4.6V , AUTO mode
    const PWCTR1_PARAMS: [u8; 3] = [0xA2, 0x02, 0x84];
    // VGH25 = 2.4C VGSEL = -10 VGH = 3 * AVDD
    const PWCTR2_PARAM: u8 = 0xC5;
    // Opamp current small, Boost frequency
    const PWCTR3_PARAMS: [u8; 2] = [0x0A, 0x00];
    // BCLK/2, Opamp current small & Medium low
    const PWCTR4_PARAMS: [u8; 2] = [0x8A, 0x2A];

    const PWCTR5_PARAMS: [u8; 2] = [0x8A, 0xEE];

    const VMCTR1_PARAM: u8 = 0x0E;

    // "Magical unicorn dust"
    const GMCTRP1_PARAMS: Option<&'static [u8; 16]> = Some(&[
        0x02, 0x1C, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2D, 0x29, 0x25, 0x2B, 0x39, 0x00, 0x01, 0x03,
        0x10,
    ]);
    // "Sparkles and rainbows"
    const GMCTRN1_PARAMS: Option<&'static [u8; 16]> = Some(&[
        0x03, 0x1D, 0x07, 0x06, 0x2E, 0x2C, 0x29, 0x2D, 0x2E, 0x2E, 0x37, 0x3F, 0x00, 0x00, 0x02,
        0x10,
    ]);
}

/// 1.77 inch TFT 128x128 from polcd
pub struct P144H008_V2;

impl MipidcsSpec for P144H008_V2 {
    const WIDTH: u16 = 128;
    const HEIGHT: u16 = 128;
    const COL_OFFSET: u16 = 0;
    const ROW_OFFSET: u16 = 0;

    const INVERTED: bool = false;
    const BGR: bool = true;
}

impl St7735Spec for P144H008_V2 {
    // 80Hz
    const FRMCTR1_PARAMS: [u8; 3] = [0x02, 0x35, 0x36];
    const FRMCTR2_PARAMS: [u8; 3] = [0x02, 0x35, 0x36];
    const FRMCTR3_PARAMS: [u8; 6] = [0x02, 0x35, 0x36, 0x02, 0x35, 0x36];

    // Dot inversion
    const INVCTR_PARAM: u8 = 0x03;

    const PWCTR1_PARAMS: [u8; 3] = [0xA2, 0x02, 0x84];
    const PWCTR2_PARAM: u8 = 0xC5;
    const PWCTR3_PARAMS: [u8; 2] = [0x0D, 0x00];
    const PWCTR4_PARAMS: [u8; 2] = [0x8D, 0x2A];
    const PWCTR5_PARAMS: [u8; 2] = [0x8D, 0xEE];

    const VMCTR1_PARAM: u8 = 0x0A;

    const GMCTRP1_PARAMS: Option<&'static [u8; 16]> = Some(&[
        0x12, 0x1C, 0x10, 0x18, 0x33, 0x2C, 0x25, 0x28, 0x28, 0x27, 0x2F, 0x3C, 0x00, 0x03, 0x03,
        0x10,
    ]);
    const GMCTRN1_PARAMS: Option<&'static [u8; 16]> = Some(&[
        0x12, 0x1C, 0x10, 0x18, 0x2D, 0x28, 0x23, 0x28, 0x28, 0x26, 0x2F, 0x3B, 0x00, 0x03, 0x03,
        0x10,
    ]);
}

/// 1.77 inch TFT 128x160
pub struct CL177SPI;

impl MipidcsSpec for CL177SPI {
    const WIDTH: u16 = 128;
    const HEIGHT: u16 = 160;
    const COL_OFFSET: u16 = 2;
    const ROW_OFFSET: u16 = 1;

    const INVERTED: bool = true;
    const BGR: bool = true;
}

impl St7735Spec for CL177SPI {
    // Frame Rate Control 1 (Normal Mode) - 0xB1
    const FRMCTR1_PARAMS: [u8; 3] = [0x00, 0x2C, 0x2B];
    // Frame Rate Control 2 (Idle Mode) - 0xB2
    const FRMCTR2_PARAMS: [u8; 3] = [0x00, 0x01, 0x01];
    // Frame Rate Control 3 (Partial Mode) - 0xB3
    const FRMCTR3_PARAMS: [u8; 6] = [0x00, 0x01, 0x01, 0x00, 0x01, 0x01];

    // Display Inversion Control - 0xB4
    const INVCTR_PARAM: u8 = 0x03;

    // Power Control 1 - 0xC0
    const PWCTR1_PARAMS: [u8; 3] = [0xA2, 0x02, 0x84];
    // Power Control 2 - 0xC1
    const PWCTR2_PARAM: u8 = 0x02;
    // Power Control 3 - 0xC2
    const PWCTR3_PARAMS: [u8; 2] = [0x0A, 0x00];
    // Power Control 4 - 0xC3
    const PWCTR4_PARAMS: [u8; 2] = [0x8A, 0x2A];
    // Power Control 5 - 0xC4
    const PWCTR5_PARAMS: [u8; 2] = [0x8A, 0xEE];

    // VCOM Control 1 - 0xC5
    const VMCTR1_PARAM: u8 = 0x09;

    // Gamma Positive - 0xE0
    const GMCTRP1_PARAMS: Option<&'static [u8; 16]> = Some(&[
        0x0C, 0x1C, 0x1B, 0x1A, 0x2F, 0x28, 0x20, 0x24, 0x23, 0x22, 0x2A, 0x36, 0x00, 0x05, 0x00,
        0x10,
    ]);
    // Gamma Negative - 0xE1
    const GMCTRN1_PARAMS: Option<&'static [u8; 16]> = Some(&[
        0x0C, 0x1A, 0x1A, 0x1A, 0x2E, 0x27, 0x21, 0x24, 0x24, 0x22, 0x2A, 0x35, 0x00, 0x05, 0x00,
        0x10,
    ]);
}
