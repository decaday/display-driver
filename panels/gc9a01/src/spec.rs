use display_driver_mipidcs::PanelSpec;

/// Specification for GC9A01 initialization differences.
pub trait Gc9a01Spec: PanelSpec {}

/// Generic 240x240 GC9A01 Panel
pub struct Generic240x240Type1;

impl PanelSpec for Generic240x240Type1 {
    const PHYSICAL_WIDTH: u16 = 240;
    const PHYSICAL_HEIGHT: u16 = 240;
    const BGR: bool = true;
    const INVERTED: bool = false;
}

impl Gc9a01Spec for Generic240x240Type1 {}

/// Generic 128x128 GC9A01 Panel
pub struct Generic128x128Type1;

impl PanelSpec for Generic128x128Type1 {
    const PHYSICAL_WIDTH: u16 = 128;
    const PHYSICAL_HEIGHT: u16 = 128;

    const PHYSICAL_X_OFFSET: u16 = 2;
    const PHYSICAL_Y_OFFSET: u16 = 1;
    const BGR: bool = true;
    const INVERTED: bool = false;
}

impl Gc9a01Spec for Generic128x128Type1 {}
