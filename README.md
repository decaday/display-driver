# display-driver

An Async display driver framework designed to provide a unified interface for various LCD panels.

This framework provides a unified interface for various LCD panels, abstracting over both simple (SPI, I2C) and complex (MIPI DSI, QSPI) display buses.
It distinguishes between command/parameter data and pixel data, allowing for atomic operations and detailed frame control (via Metadata).
This design enables efficient buffer-based operations while supporting advanced 2D graphics hardware acceleration where available.

This framework built on top of `DisplayBus` and `Panel`. `DisplayBus` provides a unified interface for various buses (SPI, I2C, MIPI DSI, QSPI, etc.), while `Panel` provides a unified interface for various panels (ST7789, GC9A01, etc.).The `DisplayDriver` struct combines them to provide a high-level interface for drawing.

## Display Panel Implementations

- [mipidcs](./mipidcs): Common impl for standard MIPI DCS.

- [co5300](./panels/co5300): CO5300, common used on AMOLED.


## License

This project is under Apache License, Version 2.0 ([LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).
