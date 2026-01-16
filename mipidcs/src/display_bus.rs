use display_driver::bus::DisplayBus;
use display_driver::panel::{reset::LCDReseter, Orientation, Panel, initseq::sequenced_init};

use display_driver::{ColorFormat, DisplayError};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use crate::{DisplaySize, GenericMipidcs};
use crate::consts::*;
use crate::dcs_types::*;


impl<B, S, RST> Panel<B> for GenericMipidcs<B, S, RST>
where
    B: DisplayBus,
    S: DisplaySize,
    RST: OutputPin,
{
    const CMD_LEN: usize = 1;
    const X_ALIGNMENT: u16 = 1;
    const Y_ALIGNMENT: u16 = 1;

    async fn init<D: DelayNs>(&mut self, bus: &mut B, mut delay: D) -> Result<(), B::Error> {
        // Hardware Reset
        let mut reseter = LCDReseter::new(&mut self.reset_pin, bus, &mut delay, 10);
        reseter.reset().await?;

        sequenced_init(Self::INIT_STEPS.into_iter(), &mut delay, bus).await?;
        self.set_address_mode(bus, self.address_mode).await
    }

    fn size(&self) -> (u16, u16) {
        (S::WIDTH, S::HEIGHT)
    }

    async fn set_window(
        &mut self,
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), DisplayError<B::Error>> {
        let x_start = x0 + S::COL_OFFSET;
        let x_end = x1 + S::COL_OFFSET;
        let y_start = y0 + S::ROW_OFFSET;
        let y_end = y1 + S::ROW_OFFSET;

        self.set_column_address(bus, x_start, x_end).await.map_err(DisplayError::BusError)?;
        self.set_page_address(bus, y_start, y_end).await.map_err(DisplayError::BusError)
    }

    fn pixel_write_command(&mut self) -> [u8; 4] {
        [WRITE_MEMORY_START, 0, 0, 0]
    }

    async fn set_color_format(
        &mut self,
        bus: &mut B,
        color_format: ColorFormat,
    ) -> Result<(), DisplayError<B::Error>> {
        let bits = color_format.size_bits();
        
        // Use from_bit_count as requested
        let pf_type = PixelFormatType::from_bit_count(bits)
            .ok_or(DisplayError::Unsupported)?;
            
        // Use dbi_and_dpi for better compatibility
        let pf = PixelFormat::dbi_and_dpi(pf_type);

        self.set_pixel_format(bus, pf)
            .await
            .map_err(DisplayError::BusError)
    }

    async fn set_orientation(
        &mut self,
        bus: &mut B,
        orientation: Orientation,
    ) -> Result<(), DisplayError<B::Error>> {
        let mut mode = self.address_mode;

        // Calculate new orientation bits
        let (mx, my, mv) = match orientation {
            Orientation::Deg0 => (false, false, false),
            Orientation::Deg90 => (true, false, true),
            Orientation::Deg180 => (true, true, false),
            Orientation::Deg270 => (false, true, true),
        };

        mode.set(AddressMode::MX, mx);
        mode.set(AddressMode::MY, my);
        mode.set(AddressMode::MV, mv);

        self.address_mode = mode;
        self.set_address_mode(bus, mode)
            .await
            .map_err(DisplayError::BusError)
    }
}