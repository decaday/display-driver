# CO5300 AMOLED Display Driver

This crate provides a driver for the CO5300 AMOLED display controller, often found in high-resolution wearables (e.g., 454x454 or 466x466).

![sf32-slint-co5300-amoled](../../docs/sf32-slint-co5300-amoled.jpg)

## Constraints

> [!WARNING]
> **Address Window Alignment**
> The CO5300 controller requires 2-byte alignment for column and row addresses.
> *   `x` and `y` start coordinates must be even.
> *   Width and height must be multiples of 2.
> *   When writing pixel data, always write **at least 2 lines**.
>
> Failure to follow this will result in visual artifacts or the display ignoring the command.


## Usage

This driver is designed to work with the [display-driver crate](https://github.com/decaday/display-driver).

### 1. Choose a Spec

The CO5300 controller powers various high-resolution AMOLED panels. These panels differ in resolution, offsets, and initialization sequences. 
This crate uses a **Spec** trait (`Co5300Spec`) to handle these differences at compile time.

You must choose a Spec that matches your hardware from the [Built-in Panel Specs](#built-in-panel-specs) list, or implement your own.

### 2. Implementation Example

```rust
use display_driver::{ColorFormat, DisplayDriver, Orientation, LCDResetOption, FrameControl, Area};
use dd_co5300::{Co5300, spec::AM196Q410502LK_196};

// ... Setup SPI/QSPI bus and Reset Pin ...

// Configure Reset
let reset_opt = LCDResetOption::new_pin(reset_pin);

// Create the Panel instance using a Spec (e.g., AM196Q410502LK_196)
let panel = Co5300::<AM196Q410502LK_196, _, _>::new(reset_opt);

// Create the DisplayDriver
// The driver orchestrates the logic, delegating transport to 'bus' and commands to 'panel'.
let mut display = DisplayDriver::new(bus, panel);

// Async Initialization
display.init(&mut delay).await.unwrap();
display.set_color_format(ColorFormat::RGB565).await.unwrap();
display.set_orientation(Orientation::Portrait).await.unwrap();
display.panel().set_brightness(display.bus(), 200).await.unwrap();

// Draw
display.write_pixels(
    Area::from_origin(100, 100),
    FrameControl::new_first(),
    partial_buffer,
);
```

For a complete working example using Slint UI on the SF32LB52x platform, please visit:
[**SF32 Slint Example**](https://github.com/decaday/sf32-slint-example)


## Panel Specs
### Built-in Panel Specs

The following panels are defined in `src/spec.rs`. Use the corresponding struct as the `Spec` generic parameter when initializing `Co5300`.

| Struct Name | Resolution | Offsets (X, Y) | Note |
| :--- | :--- | :--- | :--- |
| `AM196Q410502LK_196` | 410x502 | (22, 0) | - |
| `AM178Q368448LK_178` | 368x448 | (16, 0) | - |
| `AM151Q466466LK_151_C` | 466x466 | (6, 0) | - |
| `AM200Q460460LK_200` | 460x460 | (10, 0) | - |
| `H0198S005AMT005_V0_195` | 410x502 | (44, 0) | - |
| `Amoled_185Inch_390x450` | 390x450 | (0, 0) | Uses Page Switch 0x20 |
| `GenericCo5300` | 240x240 | (0, 0) | Default placeholder |

### Implementing a Custom Spec

To support a new panel, implement the `Co5300Spec` trait.

```rust
use dd_co5300::{Co5300Spec, MipidcsSpec};

pub struct MyNewPanel;

impl MipidcsSpec for MyNewPanel {
    const PHYSICAL_WIDTH: u16 = 454;
    const PHYSICAL_HEIGHT: u16 = 454;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;
}

impl Co5300Spec for MyNewPanel {
    /// Parameter for `REG_CMD_PAGE_SWITCH` during initialization.
    /// Try 0x00 first, then 0x20 if it doesn't work.
    const INIT_PAGE_PARAM: u8 = 0x00;

    /// Force `read_id` to succeed regardless of hardware response.
    /// But actually, we haven't implemented `read_id` yet.
    const IGNORE_ID_CHECK: bool = true;
}
```

## License

This project is under Apache License, Version 2.0 ([LICENSE](../../LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).