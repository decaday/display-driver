# ST7735 Display Driver

This crate provides an async driver for the ST7735 display controller, implementing the `Panel` trait to be used with the `display-driver` crate.

## Usage

This driver is intended to be used as a `Panel` implementation within the `display-driver` framework. You should not use this crate directly to control the display; instead, use it to configure a `DisplayDriver` instance.

### 1. Choose a Spec
The ST7735 has many variations (often distinguished by "Tab" colors in other libraries). This crate provides `Generic` types that map to these common variations.

### 2. Implementation Example

```rust
use display_driver::{ColorFormat, DisplayDriver, Orientation, LCDResetOption};
use dd_st7735::{St7735, spec::generic::Generic128_160Type1};

// ... Acquire your hardware resources (bus, delay, reset_pin) ...
// 'bus' must implement the `DisplayBus` trait from `display-driver`
// 'reset_pin' must implement `OutputPin`

// 1. Configure the reset option (e.g., using a physical pin)
let reset_opt = LCDResetOption::new_pin(reset_pin);

// 2. Create the Panel instance using a Generic Spec (e.g., Generic128_160Type1)
// We specify the Spec type explicitly, other generics (RST, B) are inferred
let panel = St7735::<Generic128_160Type1, _, _>::new(reset_opt);

// 3. Initialize the DisplayDriver with the bus and panel
let mut display = DisplayDriver::new(bus, panel);

// 4. Run the initialization sequence
display.init(&mut delay).await.unwrap();
display.set_color_format(ColorFormat::RGB565).await.unwrap();
display.set_orientation(Orientation::Deg90).await.unwrap();

// Now you can use `display` to draw:
// display.write_frame(fb).await.unwrap();
```

## Specs

Because ST7735 panels come in various resolutions, offsets, and color formats, we use a `Spec` trait.
- **Generic Specs**: Pre-defined configurations for common panel variations (found in `spec::generic`).
- **Vendor Specs**: Specific implementations for known physical display modules (found in `spec::vendor_specs`).

You can also define your own Spec by implementing the `St7735Spec` trait if your panel has unique requirements.

## Mapping Table

The following table maps the provided `Generic` types to common names used in the Adafruit and TFT_eSPI libraries, as well as specific vendor implementations present in this crate.

| Generic Type | Adafruit Name | TFT_eSPI Name | Vendor Specs |
| :--- | :--- | :--- | :--- |
| `Generic128_160Type1` | RedTab | RedTab | - |
| `Generic128_160Type2` | BLACKTAB | BLACKTAB | - |
| `Generic128_160Type3` | GREENTAB | GREENTAB | `CL177SPI` (1.77") |
| `Generic128_160Type4` | - | GREENTAB2 | - |
| `Generic128_160Type5` | - | GREENTAB3 | - |
| `Generic80_160_Type1` | MINI160x80 | - | - |
| `Generic80_160_Type2` | - | REDTAB160x80 | - |
| `Generic80_160_Type3` | MINI160x80PLUGIN | GREENTAB160x80 | `XX096T_IF09` (0.96") |
| `Generic128_128_Type1` | 144GREENTAB / HALLOWING | - | - |
| `Generic128_128_Type2` | - | GREENTAB128 | `P144H008_V2` (1.44") |
