#![no_main]
#![no_std]

//! CO5300 240x240 QSPI AMOLED Display Example for RP2040 using PIO
//!
//! Drives a CO5300 AMOLED display over hardware QSPI in 1-1-4 mode using RP2040 PIO + DMA.
//!
//! Pin Mapping:
//! - CLK : GPIO 0
//! - D0  : GPIO 1
//! - D1  : GPIO 2
//! - D2  : GPIO 3
//! - D3  : GPIO 4
//! - RST : GPIO 5
//! - CS  : GPIO 6

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_rp::gpio::{Level, Output};
use embassy_rp::pio::Pio;

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    geometry::{Point, Size},
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};

use display_driver::{
    eg::FrameBufferedDisplayDriver, panel::reset::LCDResetOption, ColorFormat, DisplayDriver, FrameControl,
};
use display_driver_co5300::{spec::GenericCo5300, Co5300};
use display_driver_qspi::{QspiConfig, QspiDisplayBus};
use rp2040_examples::qspi::PioQspiDevice;
use static_cell::StaticCell;

const WIDTH: usize = 240;
const HEIGHT: usize = 240;

type FramebufferType =
    Framebuffer<Rgb565, RawU16, BigEndian, WIDTH, HEIGHT, { buffer_size::<Rgb565>(WIDTH, HEIGHT) }>;

static FB: StaticCell<FramebufferType> = StaticCell::new();

embassy_rp::bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<embassy_rp::peripherals::DMA_CH0>,
                 embassy_rp::dma::InterruptHandler<embassy_rp::peripherals::DMA_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("=== CO5300 QSPI Display Example (RP2040 PIO) ===");

    let p = embassy_rp::init(Default::default());

    // ── Pin assignments ────────────────────────────────────────────────────
    // CLK: GPIO 0, D0: GPIO 1, D1: GPIO 2, D2: GPIO 3, D3: GPIO 4
    // RST: GPIO 5, CS: GPIO 6
    let cs = Output::new(p.PIN_6, Level::High);
    let rst = Output::new(p.PIN_5, Level::High);

    info!("Initializing PIO0 QSPI Device...");
    let pio = Pio::new(p.PIO0, Irqs);
    let mut common = pio.common;
    let sm0 = pio.sm0;

    let qspi_device = PioQspiDevice::new(
        &mut common,
        sm0,
        p.PIN_0, // SCK
        p.PIN_1, // D0
        p.PIN_2, // D1
        p.PIN_3, // D2
        p.PIN_4, // D3
        cs,
        p.DMA_CH0,
        p.DMA_CH1,
        Irqs,
        10_000_000, // 10 MHz QSPI clock
    );

    // Default QspiConfig: cmd_data_mode=Single, ram_data_mode=Quad (1-1-4)
    let bus = QspiDisplayBus::new(qspi_device, QspiConfig::default());

    let panel = Co5300::<GenericCo5300, _, _>::new(LCDResetOption::new_pin(rst));

    info!("Initializing DisplayDriver...");
    let disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();

    info!("Display initialized.");

    // Initialize framebuffer
    let fb = FB.init(Framebuffer::new());
    let mut fb_disp = FrameBufferedDisplayDriver::new(disp, fb);

    // Set brightness
    fb_disp.set_brightness(200).await.unwrap();
    info!("Brightness set to 200.");

    // ── Draw demo scene ──────────────────────────────────────────
    fb_disp.clear(Rgb565::BLACK).unwrap();

    // Three overlapping coloured shapes
    Rectangle::new(Point::new(10, 40), Size::new(100, 100))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(&mut fb_disp)
        .unwrap();

    Rectangle::new(Point::new(70, 70), Size::new(100, 100))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(&mut fb_disp)
        .unwrap();

    Circle::new(Point::new(110, 110), 110)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
        .draw(&mut fb_disp)
        .unwrap();

    // Title text
    let title = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);
    Text::with_alignment("CO5300 QSPI", Point::new(120, 22), title, Alignment::Center)
        .draw(&mut fb_disp)
        .unwrap();

    // Subtitle
    let sub = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::CYAN);
    Text::with_alignment("RP2040 PIO display-driver", Point::new(120, 232), sub, Alignment::Center)
        .draw(&mut fb_disp)
        .unwrap();

    info!("Flushing framebuffer to display...");

    // Chunked flush (4 chunks of 60 lines)
    for chunk in 0..4 {
        let start_line = chunk * 60;
        let end_line = start_line + 59;

        let frame_ctrl = FrameControl {
            first: chunk == 0,
            last: chunk == 3,
        };

        fb_disp
            .flush_lines_with_frame_control(start_line as u16, end_line as u16, frame_ctrl)
            .await
            .unwrap();
    }

    info!("Done! Display should show the demo scene.");

    let mut ticks = 0;
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1000)).await;
        ticks += 1;
        info!("Tick {}...", ticks);
    }
}
