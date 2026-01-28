/// No Operation.
///
/// Parameters: 0
pub const NOP: u8 = 0x00;

/// Software Reset.
///
/// Parameters: 0
pub const SOFT_RESET: u8 = 0x01;

/// Get the current compression mode.
///
/// Parameters: 1
pub const GET_COMPRESSION_MODE: u8 = 0x03;

/// Get the red component of the pixel at (0, 0).
///
/// Parameters: 1
pub const GET_RED_CHANNEL: u8 = 0x06;

/// Get the green component of the pixel at (0, 0).
///
/// Parameters: 1
pub const GET_GREEN_CHANNEL: u8 = 0x07;

/// Get the blue component of the pixel at (0, 0).
///
/// Parameters: 1
pub const GET_BLUE_CHANNEL: u8 = 0x08;

/// Get the current power mode.
///
/// Parameters: 1
pub const GET_POWER_MODE: u8 = 0x0a;

/// Get the data order for transfers from the Host to the display module and from the frame memory to the display device.
///
/// Parameters: 1
pub const GET_ADDRESS_MODE: u8 = 0x0b;

/// Get the current pixel format.
///
/// Parameters: 1
pub const GET_PIXEL_FORMAT: u8 = 0x0c;

/// Get the current display mode from the peripheral.
///
/// Parameters: 1
pub const GET_DISPLAY_MODE: u8 = 0x0d;

/// Get display module signaling mode.
///
/// Parameters: 1
pub const GET_SIGNAL_MODE: u8 = 0x0e;

/// Get Peripheral Self Diagnostic Result.
///
/// Parameters: 1
pub const GET_DIAGNOSTIC_RESULT: u8 = 0x0f;

/// Power for the display panel is off.
///
/// Parameters: 0
pub const ENTER_SLEEP_MODE: u8 = 0x10;

/// Power for the display panel is on.
///
/// Parameters: 0
pub const EXIT_SLEEP_MODE: u8 = 0x11;

/// Part of the display area is used for image display.
///
/// Parameters: 0
pub const ENTER_PARTIAL_MODE: u8 = 0x12;

/// The whole display area is used for image display.
///
/// Parameters: 0
pub const ENTER_NORMAL_MODE: u8 = 0x13;

/// Displayed image colors are not inverted.
///
/// Parameters: 0
pub const EXIT_INVERT_MODE: u8 = 0x20;

/// Displayed image colors are inverted.
///
/// Parameters: 0
pub const ENTER_INVERT_MODE: u8 = 0x21;

/// Selects the gamma curve used by the display device.
///
/// Parameters: 1
pub const SET_GAMMA_CURVE: u8 = 0x26;

/// Blanks the display device.
///
/// Parameters: 0
pub const SET_DISPLAY_OFF: u8 = 0x28;

/// Show the image on the display device.
///
/// Parameters: 0
pub const SET_DISPLAY_ON: u8 = 0x29;

/// Set the column extent.
///
/// Parameters: 4
pub const SET_COLUMN_ADDRESS: u8 = 0x2a;

/// Set the page extent.
///
/// Parameters: 4
pub const SET_PAGE_ADDRESS: u8 = 0x2b;

/// Transfer image data from the Host Processor to the peripheral starting at the location provided by set_column_address and set_page_address.
///
/// Parameters: variable
pub const WRITE_MEMORY_START: u8 = 0x2c;

/// Fills the peripheral look up table with the provided data.
///
/// Parameters: variable
pub const WRITE_LUT: u8 = 0x2d;

/// Transfer image data from the peripheral to the Host Processor interface starting at the location provided by set_column_address and set_page_address.
///
/// Parameters: variable
pub const READ_MEMORY_START: u8 = 0x2e;

/// Defines the number of rows in the partial display area on the display device.
///
/// Parameters: 4
pub const SET_PARTIAL_ROWS: u8 = 0x30;

/// Defines the number of columns in the partial display area on the display device.
///
/// Parameters: 4
pub const SET_PARTIAL_COLUMNS: u8 = 0x31;

/// Defines the vertical scrolling and fixed area on display device.
///
/// Parameters: 6
pub const SET_SCROLL_AREA: u8 = 0x33;

/// Synchronization information is not sent from the display module to the host processor.
///
/// Parameters: 0
pub const SET_TEAR_OFF: u8 = 0x34;

/// Synchronization information is sent from the display module to the host processor at the start of VFP.
///
/// Parameters: 1
pub const SET_TEAR_ON: u8 = 0x35;

/// Set the data order for transfers from the Host to the display module and from the frame memory to the display device.
///
/// Parameters: 1
pub const SET_ADDRESS_MODE: u8 = 0x36;

/// Defines the vertical scrolling starting point.
///
/// Parameters: 2
pub const SET_SCROLL_START: u8 = 0x37;

/// Full color depth is used on the display panel.
///
/// Parameters: 0
pub const EXIT_IDLE_MODE: u8 = 0x38;

/// Reduced color depth is used on the display panel.
///
/// Parameters: 0
pub const ENTER_IDLE_MODE: u8 = 0x39;

/// Defines how many bits per pixel are used in the interface.
///
/// Parameters: 1
pub const SET_PIXEL_FORMAT: u8 = 0x3a;

/// Transfer image information from the Host Processor interface to the peripheral from the last written location.
///
/// Parameters: variable
pub const WRITE_MEMORY_CONTINUE: u8 = 0x3c;

/// 3D is used on the display panel.
///
/// Parameters: 2
pub const SET_3D_CONTROL: u8 = 0x3d;

/// Read image data from the peripheral continuing after the last read_memory_continue or read_memory_start.
///
/// Parameters: variable
pub const READ_MEMORY_CONTINUE: u8 = 0x3e;

/// Get display module 3D Mode.
///
/// Parameters: 2
pub const GET_3D_CONTROL: u8 = 0x3f;

/// Set VSYNC timing.
///
/// Parameters: 1
pub const SET_VSYNC_TIMING: u8 = 0x40;

/// Synchronization information is sent from the display module to the host processor when the display device refresh reaches the provided scanline.
///
/// Parameters: 2
pub const SET_TEAR_SCANLINE: u8 = 0x44;

/// Get the current scanline.
///
/// Parameters: 2
pub const GET_SCANLINE: u8 = 0x45;

/// Read the DDB from the provided location.
///
/// Parameters: variable
pub const READ_DDB_START: u8 = 0xa1;

/// Continue reading the DDB from the last read location.
///
/// Parameters: variable
pub const READ_DDB_CONTINUE: u8 = 0xa8;
