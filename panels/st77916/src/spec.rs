#![allow(non_camel_case_types)]

pub use display_driver_mipidcs::PanelSpec;

/// Display Specification Trait for ST77916
pub trait St77916Spec: PanelSpec {
    /// Positive Voltage Gamma Control (0xE0) - 14 bytes
    const GAMCTRP1_PARAMS: [u8; 14];

    /// Negative Voltage Gamma Control (0xE1) - 14 bytes
    const GAMCTRN1_PARAMS: [u8; 14];
}

/// NT150XV 1.5 inch QSPI Round Display
pub struct NT150XV;

impl PanelSpec for NT150XV {
    const PHYSICAL_WIDTH: u16 = 360;
    const PHYSICAL_HEIGHT: u16 = 360; 
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;
    const INVERTED: bool = true;
    const BGR: bool = false;
}

impl St77916Spec for NT150XV {
    const GAMCTRP1_PARAMS: [u8; 14] = [0xF0, 0x10, 0x18, 0x0D, 0x0C, 0x38, 0x3E, 0x44, 0x51, 0x39, 0x15, 0x15, 0x30, 0x34];
    const GAMCTRN1_PARAMS: [u8; 14] = [0xF0, 0x0F, 0x17, 0x0D, 0x0B, 0x07, 0x3E, 0x33, 0x51, 0x39, 0x15, 0x15, 0x30, 0x34];
}
