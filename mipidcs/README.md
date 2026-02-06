# MIPI DCS

This crate provides definitions and generic implementations for the **MIPI Display Command Set (DCS)**. 

It is designed to be the foundational building block for specific display drivers (such as ST7789, ST7735, ILI9488, etc.).

This crate is an implementation detail of the [display-driver](https://github.com/decaday/display-driver) framework.

## Generic DCS Implementation
The crate provides a generic driver structure (`GenericMipidcs`) that implements standard DCS commands like `set_address_window`, `set_pixel_format`, etc. `GenericMipidcs` also implements [display-driver](https://github.com/decaday/display-driver)'s `Panel` trait.

Driver implementers can embed `GenericMipidcs` to get standard DCS functionality out of the box.

## `PanelSpec`
To support the vast variety of display panels, this crate uses the `PanelSpec` trait. This trait defines the physical properties of a panel, allowing the generic driver to automatically handle hardware differences. 

It serves as a central configuration point for resolution, physical offsets (which often vary with rotation), and color settings such as pixel inversion and RGB/BGR ordering.

## Drivers Using This Crate

You can find the list at [display-driver](https://github.com/decaday/display-driver/blob/master/README.md#display-panel-implementations).

## License

This project is under Apache License, Version 2.0 ([LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).
