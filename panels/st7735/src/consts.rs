/// Read ID1 Value.
///
/// Parameters: 0
pub const RDID1: u8 = 0xDA;

/// Read ID2 Value.
///
/// Parameters: 0
pub const RDID2: u8 = 0xDB;

/// Read ID3 Value.
///
/// Parameters: 0
pub const RDID3: u8 = 0xDC;

/// Frame Rate Control (In normal mode/ Full colors).
///
/// Parameters: 3
pub const FRMCTR1: u8 = 0xB1;

/// Frame Rate Control (In Idle mode/ 8-colors).
///
/// Parameters: 3
pub const FRMCTR2: u8 = 0xB2;

/// Frame Rate Control (In Partial mode/ full colors).
///
/// Parameters: 6
pub const FRMCTR3: u8 = 0xB3;

/// Display Inversion Control.
///
/// Parameters: 1
pub const INVCTR: u8 = 0xB4;

/// Power Control 1.
///
/// Parameters: 3
pub const PWCTR1: u8 = 0xC0;

/// Power Control 2.
///
/// Parameters: 1
pub const PWCTR2: u8 = 0xC1;

/// Power Control 3 (in Normal mode/ Full colors).
///
/// Parameters: 2
pub const PWCTR3: u8 = 0xC2;

/// Power Control 4 (in Idle mode/ 8-colors).
///
/// Parameters: 2
pub const PWCTR4: u8 = 0xC3;

/// Power Control 5 (in Partial mode/ full-colors).
///
/// Parameters: 2
pub const PWCTR5: u8 = 0xC4;

/// VCOM Control 1.
///
/// Parameters: 1
pub const VMCTR1: u8 = 0xC5;

/// VCOM Offset Control.
///
/// Parameters: 1
pub const VMOFCTR: u8 = 0xC7;

/// Write ID2 Value.
///
/// Parameters: 1
pub const WRID2: u8 = 0xD1;

/// Write ID3 Value.
///
/// Parameters: 1
pub const WRID3: u8 = 0xD2;

/// NVM Control Status.
///
/// Parameters: 1
pub const NVFCTR1: u8 = 0xD9;

/// NVM Read Command.
///
/// Parameters: 0
pub const NVFCTR2: u8 = 0xDE;

/// NVM Write Command.
///
/// Parameters: 0
pub const NVFCTR3: u8 = 0xDF;

/// Gamma (‘+’polarity) Correction Characteristics Setting.
///
/// Parameters: 16
pub const GMCTRP1: u8 = 0xE0;

/// Gamma ‘-’polarity Correction Characteristics Setting.
///
/// Parameters: 16
pub const GMCTRN1: u8 = 0xE1;

/// Gate Pump Clock Frequency Variable.
///
/// Parameters: 1
pub const GCV: u8 = 0xFC;