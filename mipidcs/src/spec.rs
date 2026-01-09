/// Display Specification Trait.
pub trait DisplaySpec {
    /// Screen width in pixels.
    const WIDTH: u16;
    /// Screen height in pixels.
    const HEIGHT: u16;
    /// Column offset in pixels (default 0).
    const COL_OFFSET: u16 = 0;
    /// Row offset in pixels (default 0).
    const ROW_OFFSET: u16 = 0;
}