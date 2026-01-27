use mipidcs::DisplaySize;

/// Specification for ST7735 initialization differences.
pub trait St7735Spec: DisplaySize {
    /// Frame Rate Control 1 (Normal Mode) - 3 bytes
    /// Frame rate=fosc/((RTNA x 2 + 40) x (LINE + FPA + BPA +2))
    const FRMCTR1_PARAMS: [u8; 3];
    /// Frame Rate Control 2 (Idle Mode) - 3 bytes
    /// Frame rate=fosc/((RTNA x 2 + 40) x (LINE + FPA + BPA +2))
    const FRMCTR2_PARAMS: [u8; 3];
    /// Frame Rate Control 3 (Partial Mode) - 6 bytes
    /// Frame rate=fosc/((RTNA x 2 + 40) x (LINE + FPA + BPA +2))
    /// 1st parameter to 3rd parameter are used in dot inversion mode.
    /// 4th parameter to 6th parameter are used in column inversion mode.
    const FRMCTR3_PARAMS: [u8; 6];

    /// Display Inversion Control - 1 byte
    const INVCTR_PARAM: u8;

    /// Power Control 1 - 3 bytes
    const PWCTR1_PARAMS: [u8; 3];
    /// Power Control 2 - 1 byte
    const PWCTR2_PARAM: u8;
    /// Power Control 3 - 2 bytes
    const PWCTR3_PARAMS: [u8; 2];
    /// Power Control 4 - 2 bytes
    const PWCTR4_PARAMS: [u8; 2];
    /// Power Control 5 - 2 bytes
    const PWCTR5_PARAMS: [u8; 2];

    /// VCOM Control 1 - 1 byte
    const VMCTR1_PARAM: u8;

    /// Gamma Positive - 16 bytes
    const GMCTRP1_PARAMS: Option<&'static [u8; 16]>;
    /// Gamma Negative - 16 bytes
    const GMCTRN1_PARAMS: Option<&'static [u8; 16]>;
}

/// 0.96 inch TFT IPS 80x160
/// XX096T_IF09
pub struct ST7735_0_96_80x160;

impl DisplaySize for ST7735_0_96_80x160 {
    const WIDTH: u16 = 80;
    const HEIGHT: u16 = 160;
    const COL_OFFSET: u16 = 26;
    const ROW_OFFSET: u16 = 1;
}

impl St7735Spec for ST7735_0_96_80x160 {
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

/// 1.44 inch TFT 128x128
/// P144H008V2
pub struct ST7735_1_44_128x128;

impl DisplaySize for ST7735_1_44_128x128 {
    const WIDTH: u16 = 128;
    const HEIGHT: u16 = 128;
    const COL_OFFSET: u16 = 0; // TODO
    const ROW_OFFSET: u16 = 0; // TODO
}

impl St7735Spec for ST7735_1_44_128x128 {
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
