#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::{self, Spi};

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, LittleEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    text::Text,
};

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{DisplayDriver, Orientation};
use display_driver::eg::FrameBufferedDisplayDriver;
use display_driver_spi::SpiDisplayBus;
use display_driver_st7735::{spec::generic::Generic80x160Type2, spec::PanelSpec, St7735};
use static_cell::StaticCell;

const IMAGE_WIDTH: usize = 86;
#[allow(unused)]
const IMAGE_HEIGHT: usize = 64;

// Rotation 90 or 270
const SCREEN_WIDTH: usize = Generic80x160Type2::PHYSICAL_HEIGHT as _;
const SCREEN_HEIGHT: usize = Generic80x160Type2::PHYSICAL_WIDTH as _;

type FramebufferType = Framebuffer<
    Rgb565,
    RawU16,
    BigEndian,
    SCREEN_WIDTH,
    SCREEN_HEIGHT,
    { buffer_size::<Rgb565>(SCREEN_WIDTH, SCREEN_HEIGHT) },
>;

static FB: StaticCell<FramebufferType> = StaticCell::new();

embassy_rp::bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<embassy_rp::peripherals::DMA_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START");

    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Wiring (RP2040 Pico -> ST7735 160x80 display)
    // 3V3  -> VCC       | GND -> GND
    // PIN_18 -> SCL/CLK | PIN_19 -> SDA/DIN
    // PIN_17 -> CS      | PIN_16 -> DC
    // PIN_20 -> RST     | PIN_21 -> BL/LED
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
    let panel = St7735::<Generic80x160Type2, _, _>::new(LCDResetOption::new_pin(rst));

    // Create and initialize the Driver using builder
    info!("Initializing display...");
    let mut disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .with_orientation(Orientation::Deg270)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();

    info!("Display initialized.");

    // Fill screen. Actually this is optional because we use framebuffer.
    disp.fill_screen_batch::<128>(Rgb565::BLACK.into())
        .await
        .unwrap();

    // Framebuffer
    let fb = FB.init(Framebuffer::new());

    // Create the buffered display wrapper taking ownership of disp
    let mut fb_display = FrameBufferedDisplayDriver::new(disp, fb);
    fb_display.clear(Rgb565::BLACK).unwrap();

    // Draw L-shaped markers at the corners to verify offsets
    display_driver::eg::utils::LShapedMarkers::new(
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32,
        5,
        Rgb565::RED,
    )
    .draw(&mut fb_display)
    .unwrap();

    // Draw Ferris
    let image_raw: embedded_graphics::image::ImageRaw<Rgb565, LittleEndian> =
        embedded_graphics::image::ImageRaw::new(
            include_bytes!("../../../assets/ferris.raw"),
            IMAGE_WIDTH as u32,
        );

    let image = embedded_graphics::image::Image::new(
        &image_raw,
        Point {
            x: (SCREEN_WIDTH - IMAGE_WIDTH) as i32 / 2,
            y: 5,
        },
    );

    image.draw(&mut fb_display).unwrap();

    // Draw Text
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    Text::new("powered by display-driver", Point::new(5, 75), style)
        .draw(&mut fb_display)
        .unwrap();

    // Flush to display
    info!("Flushing to display...");

    fb_display.flush().await.unwrap();

    info!("Drawing finished.");

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}
