#![no_std]

use display_driver::{bus::ErrorType, SimpleDisplayBus};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;

#[derive(Debug)]
pub enum SpiDisplayBusError<E1, E2> {
    Spi(E1),
    Dc(E2),
}

pub struct SpiDisplayBus<SPI, DC> {
    spi: SPI,
    dc: DC,
}

impl<SPI, DC> SpiDisplayBus<SPI, DC> {
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }
}

impl<SPI, DC> ErrorType for SpiDisplayBus<SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    type Error = SpiDisplayBusError<SPI::Error, DC::Error>;
}

impl<SPI, DC> SimpleDisplayBus for SpiDisplayBus<SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    async fn write_cmds(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low().map_err(SpiDisplayBusError::Dc)?;
        self.spi.write(cmd).await.map_err(SpiDisplayBusError::Spi)
    }

    async fn write_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_high().map_err(SpiDisplayBusError::Dc)?;
        self.spi.write(data).await.map_err(SpiDisplayBusError::Spi)
    }
}
