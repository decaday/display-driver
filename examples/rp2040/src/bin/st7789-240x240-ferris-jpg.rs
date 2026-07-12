#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::{self, Spi};
use embassy_time::Timer;

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    pixelcolor::{
        raw::{BigEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    text::{Baseline, Text},
    mono_font::{ascii::FONT_8X13, MonoTextStyleBuilder},
};

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{Area, DisplayDriver, FrameControl};
use display_driver_spi::SpiDisplayBus;
use display_driver_st7789::{spec::generic::Generic240x240Type1, spec::PanelSpec, St7789};

use tjpgd_rs::JpegDecoder;
use tjpgd_rs::types::{PixelFormat, Scale};

// Native dimensions (Portrait 0 degree)
const P_WIDTH: usize = Generic240x240Type1::PHYSICAL_WIDTH as usize;
const P_HEIGHT: usize = Generic240x240Type1::PHYSICAL_HEIGHT as usize;

embassy_rp::bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<embassy_rp::peripherals::DMA_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START ST7789 FERRIS JPG DEMO");

    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Wiring (RP2040 -> ST7789 240x240 display)
    let dc = Output::new(p.PIN_16, Level::Low);
    let cs = Output::new(p.PIN_17, Level::High);
    let rst = Output::new(p.PIN_20, Level::High);
    let _lcd_led = Output::new(p.PIN_21, Level::High);

    let mut spi_config: spi::Config = Default::default();
    spi_config.frequency = 10_000_000;
    spi_config.phase = spi::Phase::CaptureOnFirstTransition;
    spi_config.polarity = spi::Polarity::IdleLow;

    let spi = Spi::new_txonly(p.SPI0, p.PIN_18, p.PIN_19, p.DMA_CH0, Irqs, spi_config);

    // Create the SPI Bus
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let bus = SpiDisplayBus::new(spi_device, dc);

    // Create the Panel
    let panel = St7789::<Generic240x240Type1, _, _>::new(LCDResetOption::new_pin(rst));

    // Create and initialize the Driver using builder
    info!("Initializing display...");
    let mut disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();
    info!("Display initialized.");

    // Create framebuffer on stack
    let mut fb = Framebuffer::<
        Rgb565,
        RawU16,
        BigEndian,
        P_WIDTH,
        P_HEIGHT,
        { buffer_size::<Rgb565>(P_WIDTH, P_HEIGHT) },
    >::new();

    fb.clear(Rgb565::BLACK).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_8X13)
        .text_color(Rgb565::WHITE)
        .build();

    let text = "Powered by display-driver";
    // FONT_8X13: 8 pixels wide per char. 25 chars = 200 pixels. Center is (240 - 200) / 2 = 20.
    Text::with_baseline(text, Point::new(20, 215), text_style, Baseline::Top)
        .draw(&mut fb)
        .unwrap();

    let image_data = include_bytes!("../../../assets/ferris.jpg");
    let mut reader = &image_data[..];
    let mut pool = [0u8; 3500]; // TJpgD workspace

    let mut decoder = JpegDecoder::new(&mut pool[..], &mut reader).unwrap();
    
    info!("Decoding JPEG... {}x{}", decoder.width(), decoder.height());

    let offset_x = ((240 - decoder.width()) / 2) as i32;
    // slightly shifted up to leave room for text
    let offset_y = if 240 > decoder.height() {
        ((240 - decoder.height()) / 2 - 10) as i32
    } else {
        0
    };

    decoder.decode_to_framebuffer(
        fb.data_mut(),
        240, // pitch
        240, // height
        offset_x,
        offset_y,
        PixelFormat::RGB565,
        Scale::None,
    ).unwrap();

    info!("JPEG Decoded!");

    // Send framebuffer to display. DMA can send at most some limit, splitting just in case
    let data = fb.data();
    let (first, second) = data.split_at(data.len() / 2);

    // Send first half
    disp.write_pixels(
        Area::from_origin(P_WIDTH as u16, (P_HEIGHT / 2) as u16),
        FrameControl::new_first(),
        first,
    )
    .await
    .unwrap();

    // Send second half
    disp.write_pixels(
        Area::new(
            0,
            (P_HEIGHT / 2) as u16,
            P_WIDTH as u16,
            (P_HEIGHT / 2) as u16,
        ),
        FrameControl::new_last(),
        second,
    )
    .await
    .unwrap();

    info!("Done!");

    loop {
        Timer::after_secs(1).await;
    }
}
