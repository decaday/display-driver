#![no_std]


use embassy_stm32::rcc::*;
use embassy_stm32::time::Hertz;

/// Configure RCC for STM32H7B0 with 280MHz system clock
pub fn configure_rcc() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = Some(HSIPrescaler::DIV1);
    config.rcc.csi = true;
    config.rcc.hsi48 = Some(Hsi48Config {
        sync_from_usb: true,
    });
    config.rcc.hse = Some(Hse {
        freq: Hertz(25_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV5,
        mul: PllMul::MUL112,
        divp: Some(PllDiv::DIV2),
        divq: Some(PllDiv::DIV2),
        divr: Some(PllDiv::DIV2),
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.ahb_pre = AHBPrescaler::DIV2;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV2;
    config.rcc.apb3_pre = APBPrescaler::DIV2;
    config.rcc.apb4_pre = APBPrescaler::DIV2;
    config.rcc.voltage_scale = VoltageScale::Scale0;
    config
}

/// Enable the instruction and data caches.
///
/// `#[inline(never)]` is crucial. The `cortex-m` crate uses inline assembly that can cause an
/// `out of range pc-relative fixup value` compilation error. Interestingly, it always compiles
/// successfully locally, but fails on GitHub CI.
///
/// See: https://github.com/rust-embedded/cortex-m/issues/682
#[inline(never)]
pub fn enable_cache() {
    unsafe {
        let mut scb: cortex_m::peripheral::SCB = core::mem::transmute(());
        let mut cpuid: cortex_m::peripheral::CPUID = core::mem::transmute(());
        scb.enable_icache();
        scb.enable_dcache(&mut cpuid);
    }
}

/// Disable the data cache.
///
/// See `enable_cache()` for why this is marked `#[inline(never)]`.
#[inline(never)]
pub fn disable_cache() {
    unsafe {
        let mut scb: cortex_m::peripheral::SCB = core::mem::transmute(());
        let mut cpuid: cortex_m::peripheral::CPUID = core::mem::transmute(());
        scb.disable_dcache(&mut cpuid);
    }
}
