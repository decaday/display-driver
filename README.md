# display-driver

An Async display driver framework designed to provide a unified interface for various LCD panels.

This framework provides a unified interface for various LCD panels, abstracting over both simple (SPI, I2C) and complex (MIPI DSI, QSPI) display buses.
It distinguishes between command/parameter data and pixel data, allowing for atomic operations and detailed frame control (via Metadata).
This design enables efficient buffer-based operations while supporting advanced 2D graphics hardware acceleration where available.

<img src="./docs/sf32-slint-co5300-amoled.jpg" alt="sf32-slint-co5300-amoled" style="zoom:50%;" />

This framework built on top of `DisplayBus` and `Panel`. `DisplayBus` provides a unified interface for various buses (SPI, I2C, MIPI DSI, QSPI, etc.), while `Panel` provides a unified interface for various panels (ST7789, GC9A01, etc.).The `DisplayDriver` struct combines them to provide a high-level interface for drawing.

## Display Bus Implementations

- [spi](./buses/spi): SPI bus implementation.

- SF32 LCDC: Bus Implementation for SF32LB52x LCDC Hardware.

## Display Panel Implementations

- [mipidcs](./mipidcs): Common impl for standard MIPI DCS.

- [st7735](./panels/st7735): ST7735, common used on TFT LCD.

- [co5300](./panels/co5300): CO5300, common used on AMOLED.

## Examples

check [Examples](./examples) for more.

## License

This project is under Apache License, Version 2.0 ([LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).
