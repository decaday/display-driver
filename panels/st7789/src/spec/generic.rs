use super::*;

/// Helper macro to implement St7789Spec with default generic tuning
#[macro_export]
macro_rules! impl_st7789_generic {
    ($spec:ty) => {
        impl St7789Spec for $spec {
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
    };
}

/// Generic ST7789 spec for 240x320 displays (Type 1)
/// 240x320, offset = (0, 0), Inverted = true, RGB
pub struct Generic240x320Type1;

impl MipidcsSpec for Generic240x320Type1 {
    const PHYSICAL_WIDTH: u16 = 240;
    const PHYSICAL_HEIGHT: u16 = 320;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl_st7789_generic!(Generic240x320Type1);

/// Generic ST7789 spec for 240x240 displays (Type 1)
/// 240x240, offset = (0, 0), offset_rotated = (0, 80), Inverted = true, RGB
pub struct Generic240x240Type1;

impl MipidcsSpec for Generic240x240Type1 {
    const PHYSICAL_WIDTH: u16 = 240;
    const PHYSICAL_HEIGHT: u16 = 240;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;
    const PHYSICAL_X_OFFSET_ROTATED: u16 = 0;
    const PHYSICAL_Y_OFFSET_ROTATED: u16 = 80;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl_st7789_generic!(Generic240x240Type1);

/// Generic ST7789 spec for 135x240 displays (Type 1)
/// 135x240, offset = (52, 40), offset_rotated = (53, 40), Inverted = true, RGB
pub struct Generic135x240Type1;

impl MipidcsSpec for Generic135x240Type1 {
    const PHYSICAL_WIDTH: u16 = 135;
    const PHYSICAL_HEIGHT: u16 = 240;

    // Case 0: colstart=52, rowstart=40
    const PHYSICAL_X_OFFSET: u16 = 52;
    const PHYSICAL_Y_OFFSET: u16 = 40;

    // Case 2: colstart=53, rowstart=40
    const PHYSICAL_X_OFFSET_ROTATED: u16 = 53;
    const PHYSICAL_Y_OFFSET_ROTATED: u16 = 40;
    const INVERT_TRANSPOSED_OFFSET: bool = true;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl_st7789_generic!(Generic135x240Type1);

/// Generic ST7789 spec for 240x280 displays (1.69") (Type 1)
/// 240x280, offset = (0, 20), Inverted = true, RGB
pub struct Generic240x280Type1;

impl MipidcsSpec for Generic240x280Type1 {
    const PHYSICAL_WIDTH: u16 = 240;
    const PHYSICAL_HEIGHT: u16 = 280;

    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 20;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl_st7789_generic!(Generic240x280Type1);

/// Generic ST7789 spec for 172x320 displays (1.47") (Type 1)
/// 172x320, offset = (34, 0), Inverted = true, RGB
pub struct Generic172x320Type1;

impl MipidcsSpec for Generic172x320Type1 {
    const PHYSICAL_WIDTH: u16 = 172;
    const PHYSICAL_HEIGHT: u16 = 320;

    const PHYSICAL_X_OFFSET: u16 = 34;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl_st7789_generic!(Generic172x320Type1);

/// Generic ST7789 spec for 170x320 displays (Type 1)
/// 170x320, offset = (35, 0), Inverted = true, RGB
pub struct Generic170x320Type1;

impl MipidcsSpec for Generic170x320Type1 {
    const PHYSICAL_WIDTH: u16 = 170;
    const PHYSICAL_HEIGHT: u16 = 320;

    const PHYSICAL_X_OFFSET: u16 = 35;
    const PHYSICAL_Y_OFFSET: u16 = 0;

    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl_st7789_generic!(Generic170x320Type1);
