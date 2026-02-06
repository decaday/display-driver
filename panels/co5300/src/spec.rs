#![allow(non_camel_case_types)]

pub use display_driver_mipidcs::PanelSpec;

/// Display Specification Trait.
///
/// Defines resolution, offsets, and specific initialization behaviors
pub trait Co5300Spec: PanelSpec {
    /// Parameter for `REG_CMD_PAGE_SWITCH` during initialization.
    /// `0x00` for specific panels, `0x20` for else.
    const INIT_PAGE_PARAM: u8;

    /// Whether to force `read_id` to succeed regardless of hardware response.
    /// Corresponds to the logic in `LCD_ReadID`.
    const IGNORE_ID_CHECK: bool;
}

/// AM196Q410502LK_196_410x502
pub struct AM196Q410502LK_196;
impl PanelSpec for AM196Q410502LK_196 {
    const PHYSICAL_WIDTH: u16 = 410;
    const PHYSICAL_HEIGHT: u16 = 502;
    const PHYSICAL_X_OFFSET: u16 = 22;
    const PHYSICAL_Y_OFFSET: u16 = 0;
}
impl Co5300Spec for AM196Q410502LK_196 {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

/// AM178Q368448LK_178_368x448
pub struct AM178Q368448LK_178;
impl PanelSpec for AM178Q368448LK_178 {
    const PHYSICAL_WIDTH: u16 = 368;
    const PHYSICAL_HEIGHT: u16 = 448;
    const PHYSICAL_X_OFFSET: u16 = 16;
    const PHYSICAL_Y_OFFSET: u16 = 0;
}
impl Co5300Spec for AM178Q368448LK_178 {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

/// AM151Q466466LK_151_466x466_C
pub struct AM151Q466466LK_151_C;
impl PanelSpec for AM151Q466466LK_151_C {
    const PHYSICAL_WIDTH: u16 = 466;
    const PHYSICAL_HEIGHT: u16 = 466;
    const PHYSICAL_X_OFFSET: u16 = 6;
    const PHYSICAL_Y_OFFSET: u16 = 0;
}
impl Co5300Spec for AM151Q466466LK_151_C {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

/// AM200Q460460LK_200_460x460
pub struct AM200Q460460LK_200;
impl PanelSpec for AM200Q460460LK_200 {
    const PHYSICAL_WIDTH: u16 = 460;
    const PHYSICAL_HEIGHT: u16 = 460;
    const PHYSICAL_X_OFFSET: u16 = 10;
    const PHYSICAL_Y_OFFSET: u16 = 0;
}
impl Co5300Spec for AM200Q460460LK_200 {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

/// H0198S005AMT005_V0_195_410x502
pub struct H0198S005AMT005_V0_195;
impl PanelSpec for H0198S005AMT005_V0_195 {
    const PHYSICAL_WIDTH: u16 = 410;
    const PHYSICAL_HEIGHT: u16 = 502;
    const PHYSICAL_X_OFFSET: u16 = 44;
    const PHYSICAL_Y_OFFSET: u16 = 0;
}
impl Co5300Spec for H0198S005AMT005_V0_195 {
    const INIT_PAGE_PARAM: u8 = 0x00;
    const IGNORE_ID_CHECK: bool = true;
}

// Amoled, 1.85Inch 390x450
pub struct Amoled_185Inch_390x450;
impl PanelSpec for Amoled_185Inch_390x450 {
    const PHYSICAL_WIDTH: u16 = 390;
    const PHYSICAL_HEIGHT: u16 = 450;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;
}
impl Co5300Spec for Amoled_185Inch_390x450 {
    const INIT_PAGE_PARAM: u8 = 0x20;
    const IGNORE_ID_CHECK: bool = true;
}

pub struct GenericCo5300;
impl PanelSpec for GenericCo5300 {
    const PHYSICAL_WIDTH: u16 = 240; // Default placeholder
    const PHYSICAL_HEIGHT: u16 = 240; // Default placeholder
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;
}
impl Co5300Spec for GenericCo5300 {
    const INIT_PAGE_PARAM: u8 = 0x20;
    const IGNORE_ID_CHECK: bool = true;
}
