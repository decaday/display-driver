pub use display_driver_mipidcs::PanelSpec;

pub mod generic;
pub mod vendor_specs;

/// Specification for ST7789 initialization differences.
pub trait St7789Spec: PanelSpec {
    /// Porch Setting (0xB2) - 5 bytes
    const PORCTRL_PARAMS: [u8; 5];

    /// Gate Control (0xB7) - 1 byte
    const GCTRL_PARAM: u8;

    /// VCOM Setting (0xBB) - 1 byte
    const VCOMS_PARAM: u8;

    /// LCM Control (0xC0) - 1 byte
    const LCMCTRL_PARAM: u8;

    /// VRH Set (0xC3) - 1 byte
    const VRHS_PARAM: u8;

    /// VDV Set (0xC4) - 1 byte
    const VDVS_PARAM: u8;

    /// Frame Rate Control in Normal Mode (0xC6) - 1 byte
    const FRCTRL2_PARAM: u8;

    /// Power Control 1 (0xD0) - 2 bytes
    const PWCTRL1_PARAMS: [u8; 2];

    /// Positive Voltage Gamma Control (0xE0) - 14 bytes
    const PVGAMCTRL_PARAMS: [u8; 14];

    /// Negative Voltage Gamma Control (0xE1) - 14 bytes
    const NVGAMCTRL_PARAMS: [u8; 14];

    /// Power Control 2 (0xE8) - 1 byte
    const PWCTRL2_PARAMS: Option<&'static [u8; 1]> = None;

    /// Equalize time control (0xE9) - 3 bytes
    const EQCTRL_PARAMS: Option<&'static [u8; 3]> = None;

    /// Gate Output Selection (0xD6) - 1 byte
    ///
    /// Note: Avalible in P3, Not Avalible in V, VW
    const GATESEL_PARAMS: Option<&'static [u8; 1]> = None;
}
