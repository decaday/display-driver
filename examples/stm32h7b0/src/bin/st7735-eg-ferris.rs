#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;

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

use dd_st7735::{spec::vendor_specs::XX096T_IF09, spec::MipidcsSpec, St7735};
use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{DisplayDriver, Orientation};
use display_driver_spi::SpiDisplayBus;
use static_cell::StaticCell;

const IMAGE_WIDTH: usize = 86;
const IMAGE_HEIGHT: usize = 64;

// Rotation 90 or 270
const SCREEN_WIDTH: usize = XX096T_IF09::PHYSICAL_HEIGHT as _;
const SCREEN_HEIGHT: usize = XX096T_IF09::PHYSICAL_WIDTH as _;

type FramebufferType = Framebuffer<
    Rgb565,
    RawU16,
    BigEndian,
    SCREEN_WIDTH,
    SCREEN_HEIGHT,
    { buffer_size::<Rgb565>(SCREEN_WIDTH, SCREEN_HEIGHT) },
>;

static FB: StaticCell<FramebufferType> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // RCC config
    let mut config = Config::default();
    info!("START");
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        // Needed for USB
        config.rcc.hsi48 = Some(Hsi48Config {
            sync_from_usb: true,
        });
        // External oscillator 25MHZ
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
    }

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    let dc = Output::new(p.PE13, Level::Low, Speed::High);
    let cs = Output::new(p.PE11, Level::Low, Speed::High);
    let _lcd_led = Output::new(p.PE10, Level::Low, Speed::Low);

    let mut spi_config: spi::Config = Default::default();
    spi_config.frequency = Hertz(24_000_000);

    let spi = Spi::new_txonly(p.SPI4, p.PE12, p.PE14, p.DMA1_CH0, spi_config);

    // Create the SPI Bus
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let bus = SpiDisplayBus::new(spi_device, dc);

    // Create the Panel
    let panel = St7735::<XX096T_IF09, _, _>::new(LCDResetOption::new_software());

    // Create the Driver
    let mut disp = DisplayDriver::new(bus, panel);

    // Initialize
    info!("Initializing display...");
    disp.init(&mut embassy_time::Delay).await.unwrap();

    disp.set_color_format(ColorFormat::RGB565).await.unwrap();
    disp.set_orientation(Orientation::Deg270).await.unwrap();

    info!("Display initialized.");

    // Fill screen. Actually this is optional because we use framebuffer.
    disp.fill_screen_batch::<128>(Rgb565::BLACK.into())
        .await
        .unwrap();

    // Framebuffer
    let fb = FB.init(Framebuffer::new());
    fb.clear(Rgb565::BLACK);

    // Draw L-shaped markers at the corners to verify offsets
    stm32h7b0_examples::LShapedMarkers::new(
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32,
        5,
        Rgb565::RED,
    )
    .draw(fb)
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

    image.draw(fb).unwrap();

    // Draw Text
    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    Text::new("powered by display-driver", Point::new(5, 75), style)
        .draw(fb)
        .unwrap();

    // Flush to display
    info!("Flushing to display...");

    disp.write_frame(fb.data()).await.unwrap();

    info!("Drawing finished.");

    loop {}
}
