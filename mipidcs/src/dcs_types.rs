use core::mem;

use bitflags::bitflags;
use display_driver::panel::Orientation;

/// Gamma Curve selection (Command 0x26).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GammaSet {
    /// Gamma Curve 1 (G1.0)
    GC0 = 0b1,
    /// Gamma Curve 2 (G1.8)
    GC1 = 0b10,
    /// Gamma Curve 3 (G2.2)
    GC2 = 0b100,
    /// Gamma Curve 4 (G2.5)
    GC3 = 0b1000,
}

impl GammaSet {
    /// Create a GammaSet from a generic integer index (0-3).
    pub fn gc(n: u8) -> Option<Self> {
        match n {
            0..=3 => unsafe { mem::transmute(1u8 << n) },
            _ => None,
        }
    }
}

/// Pixel Format for the (Command 0x3A).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormatType {
    Bits3 = 0x1,
    Bits8 = 0x2,
    Bits12 = 0x3,
    Bits16 = 0x5,
    Bits18 = 0x6,
    Bits24 = 0x7,
}

impl PixelFormatType {
    /// Convert a raw bit count (e.g., 16) into the corresponding enum variant.
    pub const fn from_bit_count(bit_count: u8) -> Option<Self> {
        match bit_count {
            3 => Some(Self::Bits3),
            8 => Some(Self::Bits8),
            12 => Some(Self::Bits12),
            16 => Some(Self::Bits16),
            18 => Some(Self::Bits18),
            24 => Some(Self::Bits24),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelFormat(pub u8);

impl PixelFormat {
    pub const fn dbi_only(value: PixelFormatType) -> Self {
        Self(value as u8)
    }

    pub const fn dpi_only(value: PixelFormatType) -> Self {
        Self((value as u8) << 4)
    }

    pub const fn dbi_and_dpi(value: PixelFormatType) -> Self {
        Self(((value as u8) << 4) | (value as u8))
    }
}

bitflags! {
    /// Memory Data Access Control (MADCTL) flags (Command 0x36).
    ///
    /// This controls how the frame memory is accessed by the display controller,
    /// effectively handling rotation, mirroring, RGB/BGR swapping, and flip modes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AddressMode: u8 {
        /// B0 - Flip Vertical (GS)
        ///
        /// Flips the image shown on the display device top to bottom.
        /// No change is made to the frame memory contents or read order.
        /// 0 = Normal, 1 = Flipped.
        const FLIP_VERTICAL = 0b0000_0001;

        /// B1 - Flip Horizontal (SS)
        ///
        /// Flips the image shown on the display device left to right.
        /// No change is made to the frame memory.
        /// 0 = Normal, 1 = Flipped.
        const FLIP_HORIZONTAL = 0b0000_0010;

        /// B2 - Display Data Latch Order (MH)
        ///
        /// Controls the display device's horizontal line data latch order.
        /// 0 = Left to Right, 1 = Right to Left.
        const HORIZONTAL_REFRESH_ORDER = 0b0000_0100;

        /// B3 - RGB/BGR Order (BGR)
        ///
        /// Controls the RGB data latching order.
        /// 0 = RGB, 1 = BGR.
        const BGR = 0b0000_1000;

        /// B4 - Line Refresh Order (ML)
        ///
        /// Controls the vertical line refresh order of the display.
        /// 0 = Top to Bottom, 1 = Bottom to Top.
        const VERTICAL_REFRESH_ORDER = 0b0001_0000;

        /// B5 - Page/Column Address Order (MV)
        ///
        /// Controls the exchange of Row and Column addresses (X/Y Swap).
        /// 0 = Normal, 1 = Reverse (Swap X/Y).
        const ROW_COLUMN_SWAP = 0b0010_0000;

        /// B6 - Column Address Order (MX)
        ///
        /// Controls the order columns are transferred.
        /// 0 = Left to Right, 1 = Right to Left (Horizontal Mirror).
        const COLUMN_ADDRESS_ORDER = 0b0100_0000;

        /// B7 - Page Address Order (MY)
        ///
        /// Controls the order pages (rows) are transferred.
        /// 0 = Top to Bottom, 1 = Bottom to Top (Vertical Mirror).
        const PAGE_ADDRESS_ORDER = 0b1000_0000;

        // --- Common Aliases for Driver Compatibility ---

        /// Alias for Horizontal Refresh Order (B2).
        const MH = Self::HORIZONTAL_REFRESH_ORDER.bits();
        /// Alias for Vertical Refresh Order (B4).
        const ML = Self::VERTICAL_REFRESH_ORDER.bits();
        /// Alias for Row/Column Swap (B5).
        const MV = Self::ROW_COLUMN_SWAP.bits();
        /// Alias for Column Address Order (Horizontal Mirror) (B6).
        const MX = Self::COLUMN_ADDRESS_ORDER.bits();
        /// Alias for Page Address Order (Vertical Mirror) (B7).
        const MY = Self::PAGE_ADDRESS_ORDER.bits();
    }
}

impl AddressMode {
    /// Creates a simplified AddressMode for common rotation/color scenarios.
    ///
    /// This mimics the old constructor for easier migration.
    pub fn new_simple(mx: bool, my: bool, mv: bool, bgr: bool) -> Self {
        let mut mode = Self::empty();
        mode.set(AddressMode::MX, mx);
        mode.set(AddressMode::MY, my);
        mode.set(AddressMode::MV, mv);
        mode.set(AddressMode::BGR, bgr);
        mode
    }

    pub const fn as_bytes(&self) -> [u8; 1] {
        [self.bits()]
    }

    pub const fn is_xy_swapped(&self) -> bool {
        self.contains(AddressMode::MV)
    }

    pub const fn from_orientation(orientation: Orientation) -> Self {
        let bits = match orientation {
            Orientation::Deg0 => Self::empty().bits(),
            Orientation::Deg90 => Self::MV.bits() | Self::MX.bits(),
            Orientation::Deg180 => Self::MX.bits() | Self::MY.bits(),
            Orientation::Deg270 => Self::MV.bits() | Self::MY.bits(),
        };
        Self::from_bits_truncate(bits)
    }

    pub const fn orientation(&self) -> Option<Orientation> {
        let bits = self.bits();
        match bits {
            0 => Some(Orientation::Deg0),
            _ => None,
        }
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        let bits = Self::from_orientation(orientation).bits();
        let mask_bits = Self::MV.bits() | Self::MX.bits() | Self::MY.bits();
        *self = Self::from_bits_retain((self.bits() & !mask_bits) | (bits & mask_bits));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressRange(pub [u8; 4]);

impl AddressRange {
    pub const fn new(start: u16, end: u16) -> Self {
        let s = start.to_be_bytes();
        let e = end.to_be_bytes();
        Self {
            0: [s[0], s[1], e[0], e[1]],
        }
    }

    pub const fn new_with_offset(start: u16, end: u16, offset: u16) -> Self {
        let s = (start + offset).to_be_bytes();
        let e = (end + offset - 1).to_be_bytes();
        Self {
            0: [s[0], s[1], e[0], e[1]],
        }
    }

    pub const fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }
}
