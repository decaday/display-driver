# display-driver

An Async display driver framework designed to provide a unified interface for various LCD panels.

This framework provides buffer-based (like framebuffer) operations for now, and try to leverage advanced 2D graphics hardware.

## Display Panel Implementations

- [mipidcs](./panels/mipidcs): Common impl for standard MIPI DCS.

- [co5300](./panels/co5300): CO5300, common used on AMOLED.


## License

This project is under Apache License, Version 2.0 ([LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).
