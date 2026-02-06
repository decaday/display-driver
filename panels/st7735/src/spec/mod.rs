pub use display_driver_mipidcs::PanelSpec;

pub mod generic;
pub mod vendor_specs;

/// Specification for ST7735 initialization differences.
pub trait St7735Spec: PanelSpec {
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

    /// Gamma Positive - 16 bytes, Magical Unicorn Dust
    const GMCTRP1_PARAMS: Option<&'static [u8; 16]>;
    /// Gamma Negative - 16 bytes, Sparkles and Rainbows
    const GMCTRN1_PARAMS: Option<&'static [u8; 16]>;
}
