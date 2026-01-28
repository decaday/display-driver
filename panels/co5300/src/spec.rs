#![allow(non_camel_case_types)]

pub use mipidcs::DisplaySize;

/// Display Specification Trait.
///
/// Defines resolution, offsets, and specific initialization behaviors
pub trait Co5300Spec: DisplaySize {
    /// Parameter for `REG_CMD_PAGE_SWITCH` during initialization.
    /// `0x00` for specific panels, `0x20` for else.
    const INIT_PAGE_PARAM: u8;

    /// Whether to force `read_id` to succeed regardless of hardware response.
    /// Corresponds to the logic in `LCD_ReadID`.
    const IGNORE_ID_CHECK: bool;
}

/// AM196Q410502LK_196_410x502
pub struct AM196Q410502LK_196;
impl DisplaySize for AM196Q410502LK_196 {
    const WIDTH: u16 = 410;
    const HEIGHT: u16 = 502;
    const COL_OFFSET: u16 = 22;
    const ROW_OFFSET: u16 = 0;
}
impl Co5300Spec for AM196Q410502LK_196 {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

/// AM178Q368448LK_178_368x448
pub struct AM178Q368448LK_178;
impl DisplaySize for AM178Q368448LK_178 {
    const WIDTH: u16 = 368;
    const HEIGHT: u16 = 448;
    const COL_OFFSET: u16 = 16;
    const ROW_OFFSET: u16 = 0;
}
impl Co5300Spec for AM178Q368448LK_178 {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

/// AM151Q466466LK_151_466x466_C
pub struct AM151Q466466LK_151_C;
impl DisplaySize for AM151Q466466LK_151_C {
    const WIDTH: u16 = 466;
    const HEIGHT: u16 = 466;
    const COL_OFFSET: u16 = 6;
    const ROW_OFFSET: u16 = 0;
}
impl Co5300Spec for AM151Q466466LK_151_C {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

/// AM200Q460460LK_200_460x460
pub struct AM200Q460460LK_200;
impl DisplaySize for AM200Q460460LK_200 {
    const WIDTH: u16 = 460;
    const HEIGHT: u16 = 460;
    const COL_OFFSET: u16 = 10;
    const ROW_OFFSET: u16 = 0;
}
impl Co5300Spec for AM200Q460460LK_200 {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

/// H0198S005AMT005_V0_195_410x502
pub struct H0198S005AMT005_V0_195;
impl DisplaySize for H0198S005AMT005_V0_195 {
    const WIDTH: u16 = 410;
    const HEIGHT: u16 = 502;
    const COL_OFFSET: u16 = 44;
    const ROW_OFFSET: u16 = 0;
}
impl Co5300Spec for H0198S005AMT005_V0_195 {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

// Amoled, 1.85Inch 390x450
pub struct Amoled_185Inch_390x450;
impl DisplaySize for Amoled_185Inch_390x450 {
    const WIDTH: u16 = 390;
    const HEIGHT: u16 = 450;
    const COL_OFFSET: u16 = 0;
    const ROW_OFFSET: u16 = 0;
}
impl Co5300Spec for Amoled_185Inch_390x450 {
    const INIT_PAGE_PARAM: u8 = 0x20;
    const IGNORE_ID_CHECK: bool = true;
}

pub struct GenericCo5300;
impl DisplaySize for GenericCo5300 {
    const WIDTH: u16 = 240; // Default placeholder
    const HEIGHT: u16 = 240; // Default placeholder
    const COL_OFFSET: u16 = 0;
    const ROW_OFFSET: u16 = 0;
}
impl Co5300Spec for GenericCo5300 {
    const INIT_PAGE_PARAM: u8 = 0x20;
    const IGNORE_ID_CHECK: bool = true;
}
