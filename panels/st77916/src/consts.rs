//! ST77916 Command Set

/// Read Display ID
pub const RDDID: u8 = 0x04;

/// Read Display Status
pub const RDDST: u8 = 0x09;

/// Horizontal Scrolling Definition
pub const HSCRDEF: u8 = 0x43;

/// Horizontal Scroll Start Address of RAM
pub const HSCSAD: u8 = 0x47;

/// Compress On
pub const CPON: u8 = 0x4A;

/// Compress Off
pub const CPOFF: u8 = 0x4B;

/// Memory Clear Act
pub const RAMCLACT: u8 = 0x4C;

/// Memory Clear Set R
pub const RAMCLSETR: u8 = 0x4D;

/// Memory Clear Set G
pub const RAMCLSETG: u8 = 0x4E;

/// Memory Clear Set B
pub const RAMCLSETB: u8 = 0x4F;

/// CDC Control
pub const CDCCTR: u8 = 0x50;

/// Write Display Brightness
pub const WRDISBV: u8 = 0x51;

/// Read Display Brightness
pub const RDDISBV: u8 = 0x52;

/// Write CTRL Display
pub const WRCTRLD: u8 = 0x53;

/// Read CTRL Display
pub const RDCTRLD: u8 = 0x54;

/// Read ID1
pub const RDID1: u8 = 0xDA;

/// Read ID2
pub const RDID2: u8 = 0xDB;

/// Read ID3
pub const RDID3: u8 = 0xDC;

/// Command Set Ctrl 1
pub const CSC1: u8 = 0xF0;

/// Command Set Ctrl 2
pub const CSC2: u8 = 0xF1;

/// Command Set Ctrl 3
pub const CSC3: u8 = 0xF2;

/// Command Set Ctrl 4
pub const CSC4: u8 = 0xF3;

/// SPI Others Read
pub const SPIOR: u8 = 0xF4;

// ---------------------------------------------------
// System Function Command Table 2
// ---------------------------------------------------

/// VRHP Set
pub const VRHPS: u8 = 0xB0;

/// VRHN Set
pub const VRHNS: u8 = 0xB1;

/// VCOM GND SET
pub const VCOMS: u8 = 0xB2;

/// STEP SET1
pub const STEP14S: u8 = 0xB5;

/// STEP SET2
pub const STEP23S: u8 = 0xB6;

/// SVDD_SVCL_SET
pub const SBSTS: u8 = 0xB7;

/// TCON_SET
pub const TCONS: u8 = 0xBA;

/// RGB_VBP
pub const RGBVBP: u8 = 0xBB;

/// RGB_HBP
pub const RGBHBP: u8 = 0xBC;

/// RGB_SET
pub const RGBSET: u8 = 0xBD;

/// Frame Rate Control A1 in Normal Mode
pub const FRCTRA1: u8 = 0xC0;

/// Frame Rate Control A2 in Normal Mode
pub const FRCTRA2: u8 = 0xC1;

/// Frame Rate Control A3 in Normal Mode
pub const FRCTRA3: u8 = 0xC2;

/// Frame Rate Control B1 in Idle Mode
pub const FRCTRB1: u8 = 0xC3;

/// Frame Rate Control B2 in Idle Mode
pub const FRCTRB2: u8 = 0xC4;

/// Frame Rate Control B3 in Idle Mode
pub const FRCTRB3: u8 = 0xC5;

/// Power Control A1 in Normal Mode
pub const PWRCTRA1: u8 = 0xC6;

/// Power Control A2 in Normal Mode
pub const PWRCTRA2: u8 = 0xC7;

/// Power Control A3 in Normal Mode
pub const PWRCTRA3: u8 = 0xC8;

/// Power Control B1 in Idle Mode
pub const PWRCTRB1: u8 = 0xC9;

/// Power Control B2 in Idle Mode
pub const PWRCTRB2: u8 = 0xCA;

/// Power Control B3 in Idle Mode
pub const PWRCTRB3: u8 = 0xCB;

/// DSTB_DSLP
pub const DSTBDSLP: u8 = 0xCF;

/// Resolution Set 1
pub const RESSET1: u8 = 0xD0;

/// Resolution Set 2
pub const RESSET2: u8 = 0xD1;

/// Resolution Set 3
pub const RESSET3: u8 = 0xD2;

/// VCOM OFFSET SET
pub const VCMOFSET: u8 = 0xDD;

/// VCOM OFFSET NEW SET
pub const VCMOFNSET: u8 = 0xDE;

/// Positive Voltage Gamma Control
pub const GAMCTRP1: u8 = 0xE0;

/// Negative Voltage Gamma Control
pub const GAMCTRN1: u8 = 0xE1;
