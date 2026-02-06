//! GC9A01 Command Definitions
//!
//! GC9A01 has many magical fields

/// Read display identification information
pub const READ_DISPLAY_ID: u8 = 0x04;

/// Read Display Status
pub const READ_DISPLAY_STATUS: u8 = 0x09;

/// Write Display Brightness
pub const WRITE_DISPLAY_BRIGHTNESS: u8 = 0x51;

/// Write CTRL Display
pub const WRITE_CTRL_DISPLAY: u8 = 0x53;

/// Read ID1
pub const READ_ID1: u8 = 0xDA;

/// Read ID2
pub const READ_ID2: u8 = 0xDB;

/// Read ID3
pub const READ_ID3: u8 = 0xDC;

// --- Level 2 Commands ---

/// RGB Interface Signal Control
pub const RGB_INTERFACE_SIGNAL_CONTROL: u8 = 0xB0;

/// Blanking Porch Control
pub const BLANKING_PORCH_CONTROL: u8 = 0xB5;

/// Display Function Control
pub const DISPLAY_FUNCTION_CONTROL: u8 = 0xB6;

/// Tearing Effect Control
pub const TEARING_EFFECT_CONTROL: u8 = 0xBA;

/// Interface Control
pub const INTERFACE_CONTROL: u8 = 0xF6;

// --- Level 3 Commands ---

/// Frame Rate Control
pub const FRAME_RATE_CONTROL: u8 = 0xE8;

/// SPI 2DATA control
pub const SPI_2DATA_CONTROL: u8 = 0xE9;

/// Power Control 1
pub const POWER_CONTROL_1: u8 = 0xC1;

/// Power Control 2
pub const POWER_CONTROL_2: u8 = 0xC3;

/// Power Control 3
pub const POWER_CONTROL_3: u8 = 0xC4;

/// Power Control 4
pub const POWER_CONTROL_4: u8 = 0xC9;

/// Power Control 7
pub const POWER_CONTROL_7: u8 = 0xA7;

/// Inter Register Enable 1
pub const INTER_REGISTER_ENABLE_1: u8 = 0xFE;

/// Inter Register Enable 2
pub const INTER_REGISTER_ENABLE_2: u8 = 0xEF;

/// Set Gamma 1
pub const SET_GAMMA_1: u8 = 0xF0;

/// Set Gamma 2
pub const SET_GAMMA_2: u8 = 0xF1;

/// Set Gamma 3
pub const SET_GAMMA_3: u8 = 0xF2;

/// Set Gamma 4
pub const SET_GAMMA_4: u8 = 0xF3;
