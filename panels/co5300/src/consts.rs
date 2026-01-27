//! CO5300 Register Definitions

pub const SW_RESET: u8 = 0x01;
pub const LCD_ID: u8 = 0x04;
pub const DSI_ERR: u8 = 0x05;
pub const POWER_MODE: u8 = 0x0A;
pub const SLEEP_IN: u8 = 0x10;
pub const SLEEP_OUT: u8 = 0x11;
pub const PARTIAL_DISPLAY: u8 = 0x12;
pub const DISPLAY_INVERSION: u8 = 0x21;
pub const DISPLAY_OFF: u8 = 0x28;
pub const DISPLAY_ON: u8 = 0x29;
pub const CASET: u8 = 0x2A;
pub const RASET: u8 = 0x2B;
pub const WRITE_RAM: u8 = 0x2C;
pub const READ_RAM: u8 = 0x2E;
pub const PART_CASET: u8 = 0x30;
pub const PART_RASET: u8 = 0x31;
pub const TEARING_EFFECT_OFF: u8 = 0x34;
pub const TEARING_EFFECT_ON: u8 = 0x35;
pub const MADCTL: u8 = 0x36;
pub const IDLE_MODE_OFF: u8 = 0x38;
pub const IDLE_MODE_ON: u8 = 0x39;
pub const COLOR_MODE: u8 = 0x3A;
pub const CONTINUE_WRITE_RAM: u8 = 0x3C;
pub const WBRIGHT: u8 = 0x51; // Write brightness
pub const RBRIGHT: u8 = 0x52; // Read brightness
pub const WRITE_CTRL_DISPLAY: u8 = 0x53;
pub const WRHBMDISBV: u8 = 0x63;
pub const DISPLAY_MODE: u8 = 0xC2;
pub const SPI_MODE: u8 = 0xC4;
pub const PASSWD1: u8 = 0xF4;
pub const PASSWD2: u8 = 0xF5;
pub const CMD_PAGE_SWITCH: u8 = 0xFE;

// Chip IDs
pub const CHIP_ID_VAL: u32 = 0x530001;
pub const CHIP_ID_ALT: u32 = 0x331100;
