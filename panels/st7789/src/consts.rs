//! ST7789 Command Set (Based on ST7789V, ST7789VW, ST7789P3 Datasheet)

/// Read Display ID.
///
/// Parameters: 0 (Returns 3 bytes)
pub const RDDID: u8 = 0x04;

/// Read Display Status.
///
/// Parameters: 0 (Returns 4 bytes)
pub const RDDST: u8 = 0x09;

/// Write Display Brightness.
///
/// Parameters: 1
pub const WRDISBV: u8 = 0x51;

/// Read Display Brightness Value.
///
/// Parameters: 1
pub const RDDISBV: u8 = 0x52;

/// Write CTRL Display.
///
/// Parameters: 1
pub const WRCTRLD: u8 = 0x53;

/// Read CTRL Value Display.
///
/// Parameters: 1
pub const RDCTRLD: u8 = 0x54;

/// Write Content Adaptive Brightness Control and Color Enhancement.
///
/// Parameters: 1
pub const WRCACE: u8 = 0x55;

/// Read Content Adaptive Brightness Control.
///
/// Parameters: 1
pub const RDCABC: u8 = 0x56;

/// Write CABC Minimum Brightness.
///
/// Parameters: 1
pub const WRCABCMB: u8 = 0x5E;

/// Read CABC Minimum Brightness.
///
/// Parameters: 1
pub const RDCABCMB: u8 = 0x5F;

/// Read Automatic Brightness Control Self-Diagnostic Result.
///
/// Parameters: 1
pub const RDABCSDR: u8 = 0x68;

/// Gate Output Selection in Sleep In Mode.
///
/// Available in: P3
/// Not available in: V, VW
///
/// Parameters: 1
pub const GATESEL: u8 = 0xD6;

/// Read ID1.
///
/// Parameters: 0 (Returns 1 byte)
pub const RDID1: u8 = 0xDA;

/// Read ID2.
///
/// Parameters: 0 (Returns 1 byte)
pub const RDID2: u8 = 0xDB;

/// Read ID3.
///
/// Parameters: 0 (Returns 1 byte)
pub const RDID3: u8 = 0xDC;

// ---------------------------------------------------
// System Function Command Table 2
// ---------------------------------------------------

/// RAM Control.
///
/// Parameters: 2
pub const RAMCTRL: u8 = 0xB0;

/// RGB Interface Control.
///
/// Parameters: 3
pub const RGBCTRL: u8 = 0xB1;

/// Porch Setting.
///
/// Parameters: 5
pub const PORCTRL: u8 = 0xB2;

/// Frame Rate Control 1 (In partial mode/ idle colors).
///
/// Parameters: 3
pub const FRCTRL1: u8 = 0xB3;

/// Partial Control.
///
/// Parameters: 1
pub const PARCTRL: u8 = 0xB5;

/// Power Saving Control.
///
/// Available in: P3
/// Not available in: V, VW
///
/// Parameters: 1
pub const PWRSAVCTRL: u8 = 0xB6;

/// Gate Control.
///
/// Parameters: 1
pub const GCTRL: u8 = 0xB7;

/// Gate On Timing Adjustment.
///
/// Parameters: 4
pub const GTADJ: u8 = 0xB8;

/// Digital Gamma Enable.
///
/// Parameters: 1
pub const DGMEN: u8 = 0xBA;

/// VCOM Setting.
///
/// Parameters: 1
pub const VCOMS: u8 = 0xBB;

/// Power Saving Mode.
///
/// Available in: VW
/// Not available in: V, P3
///
/// Parameters: 1
pub const POWSAVE: u8 = 0xBC;

/// Display off power save.
///
/// Available in: VW
/// Not available in: V, P3
///
/// Parameters: 1
pub const DLPOFFSAVE: u8 = 0xBD;

/// LCM Control.
///
/// Parameters: 1
pub const LCMCTRL: u8 = 0xC0;

/// ID Code Setting.
///
/// Parameters: 3
pub const IDSET: u8 = 0xC1;

/// VDV and VRH Command Enable.
///
/// Parameters: 2
pub const VDVVRHEN: u8 = 0xC2;

/// VRH Set.
///
/// Parameters: 1
pub const VRHS: u8 = 0xC3;

/// VDV Set.
///
/// Available in: VW, V
/// Not available in: P3
///
/// Parameters: 1
pub const VDVS: u8 = 0xC4;

/// VCOM Offset Set.
///
/// Parameters: 1
pub const VCMOFSET: u8 = 0xC5;

/// Frame Rate Control in Normal Mode.
///
/// Parameters: 1
pub const FRCTRL2: u8 = 0xC6;

/// CABC Control.
///
/// Parameters: 1
pub const CABCCTRL: u8 = 0xC7;

/// Register Value Selection 1.
///
/// Parameters: 1
pub const REGSEL1: u8 = 0xC8;

/// Register Value Selection 2.
///
/// Parameters: 1
pub const REGSEL2: u8 = 0xCA;

/// PWM Frequency Selection.
///
/// Parameters: 1
pub const PWMFRSEL: u8 = 0xCC;

/// Power Control 1.
///
/// Parameters: 2
pub const PWCTRL1: u8 = 0xD0;

/// Enable VAP/VAN signal output.
///
/// Parameters: 1
pub const VAPVANEN: u8 = 0xD2;

/// Command 2 Enable.
///
/// Parameters: 4
pub const CMD2EN: u8 = 0xDF;

/// Positive Voltage Gamma Control.
///
/// Parameters: 14
pub const PVGAMCTRL: u8 = 0xE0;

/// Negative Voltage Gamma Control.
///
/// Parameters: 14
pub const NVGAMCTRL: u8 = 0xE1;

/// Digital Gamma Look-up Table for Red.
///
/// Parameters: 64
pub const DGMLUTR: u8 = 0xE2;

/// Digital Gamma Look-up Table for Blue.
///
/// Parameters: 64
pub const DGMLUTB: u8 = 0xE3;

/// Gate Control (2).
///
/// Parameters: 3
pub const GATECTRL: u8 = 0xE4;

/// SPI2 Enable.
///
/// Parameters: 1
pub const SPI2EN: u8 = 0xE7;

/// Power Control 2.
///
/// Parameters: 1
pub const PWCTRL2: u8 = 0xE8;

/// Equalize time control.
///
/// Parameters: 3
pub const EQCTRL: u8 = 0xE9;

/// Program Mode Control.
///
/// Parameters: 1
pub const PROMCTRL: u8 = 0xEC;

/// Program Mode Enable.
///
/// Parameters: 4
pub const PROMEN: u8 = 0xFA;

/// NVM Setting.
///
/// Parameters: 2
pub const NVMSET: u8 = 0xFC;

/// Program action.
///
/// Parameters: 2
pub const PROMACT: u8 = 0xFE;
