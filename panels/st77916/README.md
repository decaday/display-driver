# ST77916 Display Driver

[![Crates.io][badge-license]][crates]
[![Crates.io][badge-version]][crates]
[![docs.rs][badge-docsrs]][docsrs]

[badge-license]: https://img.shields.io/crates/l/display-driver-st77916?style=for-the-badge
[badge-version]: https://img.shields.io/crates/v/display-driver-st77916?style=for-the-badge
[badge-docsrs]: https://img.shields.io/docsrs/display-driver-st77916?style=for-the-badge
[crates]: https://crates.io/crates/display-driver-st77916
[docsrs]: https://docs.rs/display-driver-st77916

This crate provides a `no_std` async driver for the ST77916 display controller, implementing the [`Panel` trait](https://docs.rs/display-driver/latest/display_driver/panel/trait.Panel.html) to be used with the [display-driver](https://github.com/decaday/display-driver) crate.

The ST77916 is commonly used in round QSPI displays (e.g., 360x360 1.5-inch panels).

## Usage

This driver is designed to work with the [display-driver crate](https://github.com/decaday/display-driver). You can use the `DisplayDriver` struct to orchestrate the display.

### 1. Choose a Spec
The ST77916 driver provides built-in specs for common panels, such as `NT150XV` and `JC3636W518C`. The `Spec` defines the hardware-specific constants (like resolution, offsets, and page initialization sequences).

### 2. Implementation Example

```rust
use display_driver::{ColorFormat, DisplayDriver, Orientation, LCDResetOption};
use display_driver_st77916::{St77916, spec::NT150XV};

// 1. Configure Reset
let reset_opt = LCDResetOption::new_pin(reset_pin);

// 2. Create the Panel instance using a Spec
let panel = St77916::<NT150XV, _, _>::new(reset_opt);

// 3. Bind Bus and Panel, Configure, and Initialize
// The driver orchestrates the logic, delegating transport to 'bus' and commands to 'panel'.
let mut display = DisplayDriver::builder(bus, panel)
    .with_color_format(ColorFormat::RGB565)
    // This framework automatically handles offsets and rotation.
    .with_orientation(Orientation::Deg0)
    .init(&mut delay).await.unwrap();

// Now you can use `display` to draw frames or regions:
display.write_frame(fb).await.unwrap();
```

Full examples can be found at [examples](../../examples/README.md)

## Specs

We use a `Spec` trait to handle resolution, offsets, and heavily-customized initialization sequences differences between different ST77916 panels.

| Spec Type | Resolution | Description |
| :--- | :--- | :--- |
| `NT150XV` | 360x360 | 1.5-inch QSPI Round Display |
| `JC3636W518C` | 360x360 | 1.5-inch QSPI Round Display (Alternative Manufacturer) |

### Implementing a Custom Spec

If the built-in specs don't match your display, you can easily define your own by implementing the `PanelSpec` and `St77916Spec` traits. Because ST77916 relies heavily on different Command Pages, you will need to extract the manufacturer's initialization code and define the arrays.

```rust
use display_driver_st77916::{PanelSpec, St77916Spec};
use display_driver::panel::initseq::InitStep;

// 1. Define your type
pub struct MyCustomPanel;

// 2. Configure Resolution & Offsets
impl PanelSpec for MyCustomPanel {
    const PHYSICAL_WIDTH: u16 = 360;
    const PHYSICAL_HEIGHT: u16 = 360;
    const PHYSICAL_X_OFFSET: u16 = 0;
    const PHYSICAL_Y_OFFSET: u16 = 0;
    const INVERTED: bool = true;
    const BGR: bool = false; 
}

// 3. Implement St77916Spec for specific page configurations
impl St77916Spec for MyCustomPanel {
    // Define the sequence for Command2 Page (0xF0=0x01, 0xF1=0x01)
    const COMMAND2_PAGE: &'static [InitStep<'static>] = &[
        InitStep::CommandWithParams(0xB0, &[0x69]),
        // ... other initialization steps
    ];

    // Define the Gamma Control parameters (0xF0=0x02)
    const GAMCTRP1_PARAMS: [u8; 14] = [0xF0, 0x10, /* ... */ ];
    const GAMCTRN1_PARAMS: [u8; 14] = [0xF0, 0x0F, /* ... */ ];

    // Define the sequence for GIP Command Page (0xF0=0x10, 0xF3=0x10)
    const GIP_PAGE: &'static [InitStep<'static>] = &[
        InitStep::CommandWithParams(0xE0, &[0x08]),
        // ... other GIP steps
    ];
}
```

### What if the initialization sequence is too complex?

Drivers for screens like the ST77916 often contain a massive amount of proprietary commands, with parameters varying wildly between different panels. `display-driver-st77916` attempts to make sense of these sequences and structure them for reuse via the `St77916Spec` trait.

However, this isn't strictly necessary. You can absolutely just take the manufacturer's "black box" initialization sequence as-is and implement the [`Panel` trait](https://docs.rs/display-driver/latest/display_driver/panel/trait.Panel.html) directly yourself—it's not complex at all!

## Contributing

We welcome contributions back to the project! `display-driver` accepts screen driver implementations from any manufacturer. We welcome your Pull Requests even if you don't know the exact model name of the screen you are using.

## License

This project is under Apache License, Version 2.0 ([LICENSE](../../LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).
