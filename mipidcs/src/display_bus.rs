use display_driver::bus::DisplayBus;
use display_driver::panel::{initseq::sequenced_init, reset::LCDResetHandler, Orientation, Panel};

use display_driver::{ColorFormat, DisplayError};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use crate::consts::*;
use crate::dcs_types::*;
use crate::{GenericMipidcs, PanelSpec};

impl<B, S, RST> Panel<B> for GenericMipidcs<B, S, RST>
where
    B: DisplayBus,
    S: PanelSpec,
    RST: OutputPin,
{
    const CMD_LEN: usize = 1;
    const PIXEL_WRITE_CMD: [u8; 4] = [WRITE_MEMORY_START, 0, 0, 0];

    fn width(&self) -> u16 {
        if self.address_mode.is_xy_swapped() {
            S::PHYSICAL_HEIGHT
        } else {
            S::PHYSICAL_WIDTH
        }
    }

    fn height(&self) -> u16 {
        if self.address_mode.is_xy_swapped() {
            S::PHYSICAL_WIDTH
        } else {
            S::PHYSICAL_HEIGHT
        }
    }

    fn size(&self) -> (u16, u16) {
        if self.address_mode.is_xy_swapped() {
            (S::PHYSICAL_HEIGHT, S::PHYSICAL_WIDTH)
        } else {
            (S::PHYSICAL_WIDTH, S::PHYSICAL_HEIGHT)
        }
    }

    async fn init<D: DelayNs>(&mut self, bus: &mut B, mut delay: D) -> Result<(), B::Error> {
        // Hardware Reset
        let mut reseter = LCDResetHandler::new(
            &mut self.reset_pin,
            bus,
            &mut delay,
            10,
            120,
            Some(&[SOFT_RESET]),
        );
        reseter.reset().await?;

        sequenced_init(Self::INIT_STEPS.into_iter(), &mut delay, bus).await
    }

    async fn set_window(
        &mut self,
        bus: &mut B,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), DisplayError<B::Error>> {
        self.set_address_window(bus, x0, y0, x1, y1)
            .await
            .map_err(DisplayError::BusError)
    }

    async fn set_color_format(
        &mut self,
        bus: &mut B,
        color_format: ColorFormat,
    ) -> Result<(), DisplayError<B::Error>> {
        let bits = color_format.size_bits();

        // Use from_bit_count as requested
        let pf_type = PixelFormatType::from_bit_count(bits).ok_or(DisplayError::Unsupported)?;

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

        mode.set_orientation(orientation);

        self.set_address_mode(bus, mode, Some(orientation))
            .await
            .map_err(DisplayError::BusError)
    }
}
