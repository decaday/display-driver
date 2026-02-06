# ST7789 Display Driver

This crate provides an async driver for the ST7789 display controller, implementing the `Panel` trait to be used with the [display-driver](https://github.com/decaday/display-driver) crate.

<img src="../../docs/assets/st7789.jpg" style="zoom:50%;" />

## Usage

This driver is designed to work with the [display-driver crate](https://github.com/decaday/display-driver). You can use the `DisplayDriver` struct to drive the display.

### 1. Choose a Spec

The ST7789 has many variations in resolution and internal offsets. This crate provides `Generic` types that map to common variations, as well as specific `Vendor` specs for known modules.

### 2. Implementation Example

```rust
use display_driver::{ColorFormat, DisplayDriver, Orientation, LCDResetOption};
use display_driver_st7789::{St7789, spec::generic::Generic240x240Type1};

// 1. Configure Reset
let reset_opt = LCDResetOption::new_pin(reset_pin);

// 2. Create the Panel instance using a Spec
let panel = St7789::<Generic240x240Type1, _, _>::new(reset_opt);

// 3. Bind Bus and Panel, Configure, and Initialize
let mut display = DisplayDriver::builder(bus, panel)
    .with_color_format(ColorFormat::RGB565)
    .with_orientation(Orientation::Portrait)
    .init(&mut delay).await.unwrap();

// Now you can use `display` to draw:
display.write_frame(fb).await.unwrap();
```

Full examples can be found at [examples](../../examples/README.md)

## Specs

Because ST7789 panels come in various resolutions and offsets, we use a `Spec` trait (combining `PanelSpec` and `St7789Spec`).

### Generic Specs (`spec::generic`)

| Type | Resolution | Offset (X, Y) | Rotated Offset | Note |
| :--- | :--- | :--- | :--- | :--- |
| `Generic240x320Type1` | 240x320 | (0, 0) | (0, 0) | 2.4" |
| `Generic240x240Type1` | 240x240 | (0, 0) | (0, 80) | 1.3", 1.54" |
| `Generic135x240Type1` | 135x240 | (52, 40) | (53, 40) | 1.14" |
| `Generic240x280Type1` | 240x280 | (0, 20) | - | 1.69" |
| `Generic172x320Type1` | 172x320 | (34, 0) | - | 1.47" |
| `Generic170x320Type1` | 170x320 | (35, 0) | - | 1.9" |

### Vendor Specs (`spec::vendor_specs`)

| Type | Description |
| :--- | :--- |
| `TB154` | 1.54 inch 240x240 |
| `GMT114_02` | 1.14 inch 135x240 |
| `N114_2413THBIG01_H13` | 1.14 inch 135x240 |

## Implementing a Custom Spec

If the built-in specs don't match your display, you can define your own.

> [!TIP]
> You can use the `impl_st7789_generic!` macro to use standard initialization parameters (Gamma, Voltages, etc.) while customizing resolution and offsets.

```rust
use display_driver_st7789::{PanelSpec, St7789Spec, impl_st7789_generic};

// 1. Define your type
pub struct MyCustomPanel;

// 2. Configure Resolution & Offsets
impl PanelSpec for MyCustomPanel {
    const PHYSICAL_WIDTH: u16 = 240;
    const PHYSICAL_HEIGHT: u16 = 240;

    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;
    
    // Optional: Offset when rotated 90/270 degrees
    const PHYSICAL_X_OFFSET_ROTATED: u16 = 0;
    const PHYSICAL_Y_OFFSET_ROTATED: u16 = 80;

    const INVERTED: bool = true; // IPS panels usually need this
    const BGR: bool = false;
}

// 3. Implement St7789Spec
// Option A: Use default generic initialization parameters
impl_st7789_generic!(MyCustomPanel);

// Option B: Manual implementation for fine-tuning
// impl St7789Spec for MyCustomPanel {
//     const PORCTRL_PARAMS: [u8; 5] = ...;
//     ...
// }
```

## License

This project is under Apache License, Version 2.0 ([LICENSE](../../LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).
