use crate::{DisplayBus, DisplayError, Metadata};
use core::fmt::Debug;
use embedded_hal_async::digital::Wait;

/// Errors that can occur when using a GpioTeBus.
#[derive(Debug)]
pub enum GpioTeError<BusError, WaitError> {
    /// A timeout occurred while waiting for the TE signal.
    Timeout,
    /// An error occurred while waiting on the TE pin.
    WaitError(WaitError),
    /// An error originated from the underlying bus.
    BusError(BusError),
}

/// A bus wrapper that waits for a Tearing Effect (TE) signal before initiating a frame transfer.
///
/// This is used to implement Software TE (GPIO TE). When `write_pixels` is called
/// with `metadata.frame_control.first == true`, it awaits a rising edge on the `te_pin`
/// before passing the command and data to the underlying bus.
pub struct GpioTeBus<B, P> {
    inner_bus: B,
    te_pin: P,
}

impl<B, P> GpioTeBus<B, P> {
    /// Creates a new `GpioTeBus` wrapping an existing bus and a TE pin.
    pub fn new(bus: B, te_pin: P) -> Self {
        Self {
            inner_bus: bus,
            te_pin,
        }
    }

    /// Consumes the wrapper and returns the underlying bus and pin.
    pub fn release(self) -> (B, P) {
        (self.inner_bus, self.te_pin)
    }
}

// Delegate the basic ErrorType to the inner bus
impl<B: DisplayBus, P: Wait> crate::bus::ErrorType for GpioTeBus<B, P>
where
    P::Error: Debug,
{
    type Error = GpioTeError<B::Error, P::Error>;
}

impl<B: DisplayBus, P: Wait> DisplayBus for GpioTeBus<B, P>
where
    P::Error: Debug,
{
    async fn write_cmd(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        self.inner_bus
            .write_cmd(cmd)
            .await
            .map_err(GpioTeError::BusError)
    }

    async fn write_cmd_with_params(
        &mut self,
        cmd: &[u8],
        params: &[u8],
    ) -> Result<(), Self::Error> {
        self.inner_bus
            .write_cmd_with_params(cmd, params)
            .await
            .map_err(GpioTeError::BusError)
    }

    async fn write_pixels(
        &mut self,
        cmd: &[u8],
        data: &[u8],
        metadata: Metadata,
    ) -> Result<(), DisplayError<Self::Error>> {
        // If this is the start of a frame, wait for the TE signal.
        if metadata.frame_control.first {
            // According to MIPI DCS spec (Command 35h TEON M=0), the V-Blanking period
            // active time is represented by a low signal. The start of the V-Blanking
            // period corresponds to the falling edge. So we wait for the falling edge.
            self.te_pin
                .wait_for_falling_edge()
                .await
                .map_err(|e| DisplayError::BusError(GpioTeError::WaitError(e)))?;
        }

        self.inner_bus
            .write_pixels(cmd, data, metadata)
            .await
            .map_err(|e| e.map_bus_error(GpioTeError::BusError))
    }

    fn set_reset(&mut self, reset: bool) -> Result<(), DisplayError<Self::Error>> {
        self.inner_bus
            .set_reset(reset)
            .map_err(|e| e.map_bus_error(GpioTeError::BusError))
    }
}

// Delegate other optional traits if the inner bus implements them
impl<B: crate::bus::BusHardwareFill, P: Wait> crate::bus::BusHardwareFill for GpioTeBus<B, P>
where
    P::Error: Debug,
{
    async fn fill_solid(
        &mut self,
        cmd: &[u8],
        color: crate::SolidColor,
        area: crate::Area,
    ) -> Result<(), DisplayError<Self::Error>> {
        self.inner_bus
            .fill_solid(cmd, color, area)
            .await
            .map_err(|e| e.map_bus_error(GpioTeError::BusError))
    }
}

impl<B: crate::bus::BusRead, P: Wait> crate::bus::BusRead for GpioTeBus<B, P>
where
    P::Error: Debug,
{
    async fn read_data(
        &mut self,
        cmd: &[u8],
        params: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), DisplayError<Self::Error>> {
        self.inner_bus
            .read_data(cmd, params, buffer)
            .await
            .map_err(|e| e.map_bus_error(GpioTeError::BusError))
    }
}

impl<B: crate::bus::BusBytesIo, P: Wait> crate::bus::BusBytesIo for GpioTeBus<B, P>
where
    P::Error: Debug,
{
    async fn write_cmd_bytes(&mut self, cmd: &[u8]) -> Result<(), Self::Error> {
        self.inner_bus
            .write_cmd_bytes(cmd)
            .await
            .map_err(GpioTeError::BusError)
    }

    async fn write_data_bytes(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.inner_bus
            .write_data_bytes(data)
            .await
            .map_err(GpioTeError::BusError)
    }
}
