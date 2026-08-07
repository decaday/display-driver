# QSPI Display Bus

[![Crates.io][badge-license]][crates]
[![Crates.io][badge-version]][crates]
[![docs.rs][badge-docsrs]][docsrs]

[badge-license]: https://img.shields.io/crates/l/display-driver-qspi?style=for-the-badge
[badge-version]: https://img.shields.io/crates/v/display-driver-qspi?style=for-the-badge
[badge-docsrs]: https://img.shields.io/docsrs/display-driver-qspi?style=for-the-badge
[crates]: https://crates.io/crates/display-driver-qspi
[docsrs]: https://docs.rs/display-driver-qspi

This crate provides QSPI (Quad SPI) bus abstractions and implementations for the [display-driver](https://github.com/decaday/display-driver) framework. It bridges the gap between hardware-specific QSPI peripherals and standard display panel drivers.

## Overview

Driving QSPI displays typically requires translating standard 1-byte DCS (Display Command Set) commands into complex QSPI transactions (Instruction Phase + Address Phase + Data Phase). This crate provides two primary ways to achieve this:

1. **`QspiDevice` Trait & `QspiDisplayBus`**: A low-level hardware abstraction layer for MCU-specific QSPI peripherals.
2. **`QspiFlashBus` Decorator**: A software wrapper for standard display buses.

## 1. Hardware Abstraction: `QspiDevice` & `QspiDisplayBus`

If your MCU has a dedicated QSPI peripheral that requires explicit phase configuration (Instruction, Address, Dummy Cycles, Data), you should implement the `QspiDevice` trait.

### `QspiDevice` Trait
MCU-specific HAL wrappers implement this trait to execute a `QspiTransaction`. A transaction defines:
- Instruction Phase: (e.g., `0x02`, `0x32`)
- Address Phase: (e.g., 24-bit address)
- **Dummy Cycles
- Data Phase Line Mode: `Single`, or `Quad`.

*Examples*: Implementations of the `QspiDevice` trait for different MCU families:
- **ESP32-S3** (`esp-hal`): [`esp32s3/src/qspi.rs`](../../examples/esp32s3/src/qspi.rs)
- **STM32H7** (`embassy-stm32` OCTOSPI): [`stm32h7b0_qspi/src/qspi.rs`](../../examples/stm32h7b0_qspi/src/qspi.rs)

### `QspiDisplayBus`
Once your hardware implements `QspiDevice`, you can wrap it in a `QspiDisplayBus` along with a `QspiConfig`. This bus will automatically implement the `DisplayBus` trait, allowing it to be used directly by panel drivers (like ST77916 or CO5300).

```rust
use display_driver_qspi::{QspiDisplayBus, QspiConfig, LineMode};

// Example configuration for 1-1-4 mode (1-bit cmd/addr, 4-bit data)
let config = QspiConfig {
    cmd_data_mode: LineMode::Single, // Commands use 1-bit data
    ram_data_mode: LineMode::Quad,   // Pixel RAM uses 4-bit data
    dummy_cycles: 0,
};

// `my_hw_qspi_device` must implement `QspiDevice`
let bus = QspiDisplayBus::new(my_hw_qspi_device, config);

// Pass `bus` to DisplayDriver::builder(bus, panel)
```

#### Note on Protocol Strategy
Currently, `QspiDisplayBus` uses a fixed QSPI protocol sequence commonly found in displays like CO5300 and ST77916. This implementation currently only supports the 1-1-4 mode (1-bit Instruction, 1-bit Address, 4-bit Data).
The fixed sequences are:
- **Command Write**: Inst `0x02`
- **RAM Write**: Inst `0x32`
- **Read**: Inst `0x03`
- **Address Mapping**: A 1-byte MIPI DCS command is packed into the middle byte of a 24-bit address (`0x00_CMD_00`).

*Future Work*: As more diverse QSPI protocols (e.g., ST77903) are supported, this hardcoded strategy will be abstracted into a pluggable Trait.

## 2. Software Fallback: `QspiFlashBus` Decorator

If your hardware does not implement the `QspiDevice` trait but provides a generic `DisplayBus` (like a standard SPI bus), you can use `QspiFlashBus`. 

`QspiFlashBus` acts as a decorator that intercepts 1-byte DCS commands and translates them on-the-fly into 4-byte QSPI flash commands (Instruction + 3-byte Address) before passing them to the underlying bus. 

```rust
use display_driver_qspi::QspiFlashBus;

// `spi_bus` implements `DisplayBus`
let qspi_simulated_bus = QspiFlashBus::new(spi_bus);
```
This is useful for testing QSPI panels using standard SPI hardware.

## License

This project is under Apache License, Version 2.0 ([LICENSE](../../LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).
