#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::Timer;

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::{Line, PrimitiveStyle, Triangle},
    text::Text,
};

use dd_st7735::{spec::vendor_specs::XX096T_IF09, spec::MipidcsSpec, St7735};
use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{DisplayDriver, Orientation};
use display_driver_spi::SpiDisplayBus;

// Native dimensions (Portrait 0 degree)
const P_WIDTH: usize = XX096T_IF09::PHYSICAL_WIDTH as usize;
const P_HEIGHT: usize = XX096T_IF09::PHYSICAL_HEIGHT as usize;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // RCC config
    let mut config = Config::default();
    info!("START ROTATION DEMO");
    {
        use embassy_stm32::rcc::*;
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
    info!("Display initialized.");

    // Loop orientations
    loop {
        for rot in [
            Orientation::Deg0,
            Orientation::Deg90,
            Orientation::Deg180,
            Orientation::Deg270,
        ] {
            let rot_str = match rot {
                Orientation::Deg0 => "Deg 0",
                Orientation::Deg90 => "Deg 90",
                Orientation::Deg180 => "Deg 180",
                Orientation::Deg270 => "Deg 270",
            };
            info!("Rotating to {}", rot_str);
            disp.set_orientation(rot).await.unwrap();

            // Create framebuffer on stack and draw
            match rot {
                Orientation::Deg0 | Orientation::Deg180 => {
                    // Portrait: W=80, H=160
                    let mut fb = Framebuffer::<
                        Rgb565,
                        RawU16,
                        BigEndian,
                        P_WIDTH,
                        P_HEIGHT,
                        { buffer_size::<Rgb565>(P_WIDTH, P_HEIGHT) },
                    >::new();
                    draw_scene(&mut fb, P_WIDTH, P_HEIGHT, rot_str);
                    disp.write_frame(fb.data()).await.unwrap();
                }
                Orientation::Deg90 | Orientation::Deg270 => {
                    // Landscape: W=160, H=80
                    let mut fb = Framebuffer::<
                        Rgb565,
                        RawU16,
                        BigEndian,
                        P_HEIGHT,
                        P_WIDTH,
                        { buffer_size::<Rgb565>(P_HEIGHT, P_WIDTH) },
                    >::new();
                    draw_scene(&mut fb, P_HEIGHT, P_WIDTH, rot_str);
                    disp.write_frame(fb.data()).await.unwrap();
                }
            }

            Timer::after_secs(3).await;
        }
    }
}

fn draw_scene<D>(target: &mut D, w: usize, h: usize, rot_str: &str)
where
    D: DrawTarget<Color = Rgb565> + Dimensions,
    D::Error: core::fmt::Debug,
{
    target.clear(Rgb565::BLACK).unwrap();

    let text_style = MonoTextStyle::new(&FONT_8X13, Rgb565::WHITE);

    let cx = (w / 2) as i32;
    let cy = (h / 2) as i32;

    // Draw L-shaped markers at the corners to verify offsets
    stm32h7b0_examples::LShapedMarkers::new(w as _, h as _, 5, Rgb565::RED)
        .draw(target)
        .unwrap();

    // Draw Arrow pointing UP (Visual top of the current buffer)
    // Shaft
    Line::new(Point::new(cx, cy + 20), Point::new(cx, cy - 20))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1))
        .draw(target)
        .ok();
    // Head
    Triangle::new(
        Point::new(cx, cy - 20),
        Point::new(cx - 5, cy - 10),
        Point::new(cx + 5, cy - 10),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
    .draw(target)
    .ok();

    // Centered text roughly
    Text::new(rot_str, Point::new(10, 20), text_style)
        .draw(target)
        .ok();

    // Also draw a small triangle marker in top-right
    Triangle::new(
        Point::new((w as i32) - 10, 10),
        Point::new((w as i32) - 20, 10),
        Point::new((w as i32) - 10, 20),
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 1))
    .draw(target)
    .ok();
}
