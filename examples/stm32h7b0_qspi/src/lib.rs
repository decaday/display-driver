#![no_std]

pub mod qspi;

use embassy_stm32::rcc::*;
use embassy_stm32::time::Hertz;

/// Configure RCC for STM32H7B0 with 280MHz system clock
pub fn configure_rcc() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = Some(HSIPrescaler::Div1);
    config.rcc.csi = true;
    config.rcc.hsi48 = Some(Hsi48Config {
        sync_from_usb: true,
    });
    config.rcc.hse = Some(Hse {
        freq: Hertz(25_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll1 = Some(Pll {
        source: PllSource::Hse,
        prediv: PllPreDiv::Div5,
        mul: PllMul::Mul112,
        divp: Some(PllDiv::Div2),
        divq: Some(PllDiv::Div2),
        divr: Some(PllDiv::Div2),
    });
    config.rcc.sys = Sysclk::Pll1P;
    config.rcc.ahb_pre = AHBPrescaler::Div2;
    config.rcc.apb1_pre = APBPrescaler::Div2;
    config.rcc.apb2_pre = APBPrescaler::Div2;
    config.rcc.apb3_pre = APBPrescaler::Div2;
    config.rcc.apb4_pre = APBPrescaler::Div2;
    config.rcc.voltage_scale = VoltageScale::Scale0;
    config.rcc.mux.octospisel = mux::Fmcsel::Pll1Q;
    config
}
