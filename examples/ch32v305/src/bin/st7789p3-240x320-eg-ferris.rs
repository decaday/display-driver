#![no_std]
#![no_main]

use ch32_hal as hal;
use core::convert::Infallible;
use display_driver::{
    eg::FrameBufferedDisplayDriver, Area, BusBytesIo, ColorFormat, DisplayBus, DisplayDriver,
    DisplayError, FrameControl, LCDResetOption, Orientation, Panel,
};
use display_driver_spi::SpiDisplayBus;
use display_driver_st7789::{spec::generic::Generic240x320P3Type1, St7789};
use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    image::{Image, ImageRaw},
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10},
        MonoTextStyle,
    },
    pixelcolor::{
        raw::{BigEndian, LittleEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use hal::gpio::{Level, Output, Speed};
use hal::rcc::{AHBPrescaler, APBPrescaler, Pll, PllMul, PllPreDiv, PllSource, Sysclk};
use hal::spi::{self, Spi};
use hal::time::Hertz;
use panic_halt as _;

#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = hal::Config::default();
    // 8 MHz HSI -> 144 MHz core/PCLK1. Keep the unrelated HSPLL disabled,
    // and leave LSE off because PC14 is the panel reset pin.
    config.rcc = hal::rcc::Config {
        sys: Sysclk::PLL,
        pll_src: PllSource::HSI,
        pll: Some(Pll {
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL18,
        }),
        pllx: None,
        hspll: None,
        ahb_pre: AHBPrescaler::DIV1,
        apb1_pre: APBPrescaler::DIV1,
        apb2_pre: APBPrescaler::DIV1,
        ..Default::default()
    };
    let p = hal::init(config);
    let mut backlight = Output::new(p.PB4, Level::High, Speed::Low);
    let cs = Output::new(p.PC7, Level::High, Speed::High);
    let rs = Output::new(p.PA15, Level::High, Speed::High);
    let rst = Output::new(p.PC14, Level::High, Speed::Low);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz::mhz(36);
    spi_config.mode = embedded_hal::spi::MODE_0;
    spi_config.bit_order = spi::BitOrder::MsbFirst;
    let spi = Spi::new_txonly::<0>(p.SPI3, p.PB3, p.PB5, p.DMA2_CH2, spi_config);
    let device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let bus = SpiDisplayBus::new(device, rs);
    let panel = St7789::<Generic240x320P3Type1, _, _>::new(LCDResetOption::new_pin(rst));
    let driver = match DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .with_orientation(Orientation::Deg90)
        .init(&mut Delay)
        .await
    {
        Ok(driver) => driver,
        Err(_) => {
            backlight.set_high();
            panic!("panel initialization failed");
        }
    };
    let mut display = match render_scene(driver).await {
        Ok(display) => display,
        Err(_) => {
            backlight.set_high();
            panic!("Ferris rendering failed");
        }
    };

    let mut progress = 0;
    loop {
        if update_progress(&mut display, progress).await.is_err() {
            backlight.set_high();
            panic!("progress update failed");
        }
        // Reveal only a fully rendered scene, with the initial bar cleared.
        if progress == 0 {
            backlight.set_low();
        }
        if progress == 100 {
            Timer::after_millis(500).await;
            progress = 0;
        } else {
            Timer::after_millis(30).await;
            progress += 2;
        }
    }
}

pub(crate) const WIDTH: usize = 320;
pub(crate) const HEIGHT: usize = 240;
const STRIP_HEIGHT: usize = 16;
const BAR_X: u16 = 20;
const BAR_Y: u16 = 218;
const BAR_WIDTH: u16 = 280;
const BAR_HEIGHT: u16 = 12;

// RGB565 on the wire is big-endian; the bundled Ferris asset is little-endian.
type Strip = Framebuffer<
    Rgb565,
    RawU16,
    BigEndian,
    WIDTH,
    STRIP_HEIGHT,
    { buffer_size::<Rgb565>(WIDTH, STRIP_HEIGHT) },
>;

/// The original landscape scene, drawn in screen coordinates onto a clipped target.
pub(crate) fn draw_scene<D: DrawTarget<Color = Rgb565, Error = Infallible>>(
    target: &mut D,
) -> Result<(), Infallible> {
    target.clear(Rgb565::BLACK)?;
    let grid = PrimitiveStyle::with_stroke(Rgb565::new(4, 8, 4), 1);
    for x in (0..WIDTH).step_by(20) {
        Line::new(Point::new(x as i32, 0), Point::new(x as i32, HEIGHT as i32))
            .into_styled(grid)
            .draw(target)?;
    }
    for y in (0..HEIGHT).step_by(20) {
        Line::new(Point::new(0, y as i32), Point::new(WIDTH as i32, y as i32))
            .into_styled(grid)
            .draw(target)?;
    }
    display_driver::eg::utils::LShapedMarkers::new(WIDTH as i32, HEIGHT as i32, 10, Rgb565::RED)
        .draw(target)?;
    Circle::with_center(Point::new(WIDTH as i32 / 2, HEIGHT as i32 / 2), 160)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::BLUE, 3))
        .draw(target)?;

    let ferris: ImageRaw<Rgb565, LittleEndian> =
        ImageRaw::new(include_bytes!("../../../assets/ferris.raw"), 86);
    Image::new(
        &ferris,
        Point::new((WIDTH as i32 - 86) / 2, (HEIGHT as i32 - 64) / 2 - 20),
    )
    .draw(target)?;
    Text::new(
        "powered by display-driver",
        Point::new(35, 180),
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
    )
    .draw(target)?;
    Text::new(
        "ST7789P3 320x240 Demo",
        Point::new(100, 210),
        MonoTextStyle::new(&FONT_6X10, Rgb565::CYAN),
    )
    .draw(target)?;
    Rectangle::new(
        Point::new(BAR_X as i32, BAR_Y as i32),
        Size::new(BAR_WIDTH as u32, BAR_HEIGHT as u32),
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
    .draw(target)?;
    Ok(())
}

/// Render once using 10 KiB of RAM, rather than a 150 KiB full framebuffer.
pub(crate) async fn render_scene<B: DisplayBus, P: Panel<B>>(
    driver: DisplayDriver<B, P>,
) -> Result<DisplayDriver<B, P>, DisplayError<B::Error>> {
    let mut framebuffer = Strip::new();
    let mut display = FrameBufferedDisplayDriver::new_partial(
        driver,
        Area::new(0, 0, WIDTH as u16, STRIP_HEIGHT as u16),
        &mut framebuffer,
    )?;
    for y in (0..HEIGHT).step_by(STRIP_HEIGHT) {
        display.set_area(Area::new(0, y as u16, WIDTH as u16, STRIP_HEIGHT as u16))?;
        // Partial framebuffers use local coordinates. Translate the screen scene
        // into this strip; clipping retains the correct image rows and text.
        draw_scene(&mut display.translated(Point::new(0, -(y as i32)))).unwrap();
        display
            .flush_with_frame_control(FrameControl {
                first: y == 0,
                last: y + STRIP_HEIGHT == HEIGHT,
            })
            .await?;
    }
    Ok(display.into_inner())
}

/// Update only the bar interior; the static scene stays in panel GRAM.
pub(crate) async fn update_progress<B, P>(
    display: &mut DisplayDriver<B, P>,
    progress: u8,
) -> Result<(), DisplayError<B::Error>>
where
    B: DisplayBus + BusBytesIo,
    P: Panel<B>,
{
    let inner_width = BAR_WIDTH - 4;
    let filled = inner_width * u16::from(progress) / 100;
    if filled > 0 {
        display
            .fill_solid_batch::<640>(
                Rgb565::GREEN.into(),
                Area::new(BAR_X + 2, BAR_Y + 2, filled, BAR_HEIGHT - 4),
            )
            .await?;
    }
    if filled < inner_width {
        display
            .fill_solid_batch::<640>(
                Rgb565::BLACK.into(),
                Area::new(
                    BAR_X + 2 + filled,
                    BAR_Y + 2,
                    inner_width - filled,
                    BAR_HEIGHT - 4,
                ),
            )
            .await?;
    }
    Ok(())
}
