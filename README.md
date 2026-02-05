# display-driver

An Async display driver framework designed to provide a unified interface for various LCD panels.

<img src="./docs/sf32-slint-co5300-amoled.jpg" alt="sf32-slint-co5300-amoled" style="zoom:50%;" />

## Features

- **Async-Native:** Built from the ground up with first-class `async/await` support.

- **Bus / Interface Layer:**
  Unlike simple byte-stream interfaces, `display-driver` is designed for complex communication requirements. Features include **atomic commands, stream payload classification, ROI-aware transfers, duplex operations, and hardware-accelerated fill**.
  While this architecture is crucial for supporting advanced interfaces like MIPI DSI, QSPI, or hardware with 2D graphics acceleration, it also ensures high performance for simple buses like SPI.

- **Panel Logic Layer:**
  - **MIPI DCS Standard:** Simplifies driver implementation for common controllers (e.g., ST77xx, ILI9xxx).
  - **Zero-Cost Polymorphism:** Leverages the `Spec` trait for static configuration (e.g., Gamma curves) without runtime overhead. This system includes built-in presets (e.g., `ST7735 Generic_128x128_Type1`) while fully supporting custom Spec implementations, and automatically handles coordinate offsets across different rotations.
  - **Static Init Sequences:** Uses statically computed initialization sequences to minimize Flash/RAM usage—vital for async state machines.

## Peek

```rust
use display_driver::{ColorFormat, DisplayDriver, Orientation, LCDResetOption};
use dd_st7735::{St7735, 

// The `Spec` (Generic128_160Type1) defines the hardware-specific constants (Gamma, Voltage) typesafe-ly.
spec::generic::Generic128_160Type1};

// 1. Configure Reset
let reset_opt = LCDResetOption::new_pin(reset_pin);

// 2. Create the Panel instance using a Generic Spec (e.g., Generic128_160Type1)
let panel = St7735::<Generic128_160Type1, _, _>::new(reset_opt);

// 3. Bind Bus and Panel, Configure, and Initialize
// The driver orchestrates the logic, delegating transport to 'bus' and commands to 'panel'.
let mut display = DisplayDriver::builder(bus, panel)
    .with_color_format(ColorFormat::RGB565)
    // This framework automatically handles offsets.
    .with_orientation(Orientation::Deg90)
    .init(&mut delay).await.unwrap();

// Now you can use `display` to draw:
display.write_frame(fb).await.unwrap();
```

## Display Bus Implementations

- [spi](./buses/spi): SPI bus implementation.

- SF32 LCDC: Bus Implementation for SF32LB52x LCDC Hardware.

## Display Panel Implementations

- [mipidcs](./mipidcs): Common impl for standard MIPI DCS.

- [st7735](./panels/st7735): ST7735, commonly used in TFT LCD.

- [st7789](./panels/st7789): ST7789, commonly used in TFT LCD.

- [co5300](./panels/co5300): CO5300, commonly used in AMOLED.

## Examples

check [Examples](./examples) for more.

## License

This project is under Apache License, Version 2.0 ([LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).
