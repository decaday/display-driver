# display-driver-spi

This crate provides a `SimpleDisplayBus` implementation for SPI based displays that require a separate Data/Command (DC) pin.

## Usage

```rust
use display_driver_spi::SpiDisplayBus;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;

// let spi = ...; // implement SpiDevice
// let dc = ...;  // implement OutputPin

let bus = SpiDisplayBus::new(spi, dc);
```

This bus can then be used with any display driver that accepts a `DisplayBus` (since `SpiDisplayBus` implements `SimpleDisplayBus`, which has a blanket implementation for `DisplayBus`).
