#![no_main]
#![no_std]

//! GC9A01 240x240 Concentric Gradient Demo for RP2040
//!
//! This example demonstrates driving a round GC9A01 display with a
//! 240x240 resolution. It uses a global framebuffer in AXI SRAM (or regular SRAM for RP) to
//! avoid excessive stack usage and draws a concentric gradient pattern.

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::{self, Spi};

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    geometry::Point,
    mono_font::{ascii::FONT_9X18, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    text::{Alignment, Text},
};
use micromath::F32Ext;

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{DisplayDriver, FrameControl, Orientation};
use display_driver::eg::FrameBufferedDisplayDriver;
use display_driver_gc9a01::{spec::Generic240x240Type1, Gc9a01};
use display_driver_spi::SpiDisplayBus;
use static_cell::StaticCell;

const WIDTH: usize = 240;
const HEIGHT: usize = 240;

type FramebufferType =
    Framebuffer<Rgb565, RawU16, BigEndian, WIDTH, HEIGHT, { buffer_size::<Rgb565>(WIDTH, HEIGHT) }>;

static FB: StaticCell<FramebufferType> = StaticCell::new();

embassy_rp::bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<embassy_rp::peripherals::DMA_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START GC9A01 CONCENTRIC GRADIENT DEMO");

    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Wiring (RP2040 Pico -> GC9A01 display)
    // 3V3  -> VCC     | GND -> GND
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
    let panel = Gc9a01::<Generic240x240Type1, _, _>::new(LCDResetOption::new_pin(rst));

    // Create and initialize the Driver using builder
    info!("Initializing display...");
    let disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .with_orientation(Orientation::Deg180)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();

    info!("Display initialized.");

    // Initialize framebuffer
    let fb = FB.init(Framebuffer::new());
    
    let mut fb_display = FrameBufferedDisplayDriver::new(disp, fb);

    // Draw content
    draw_concentric_gradient(&mut fb_display);
    draw_text(&mut fb_display);

    // Flush to display
    info!("Flushing to display...");

    // Split transfer into chunks because RP2040 DMA limits are similar (65535 or larger depending on ring)
    fb_display.flush_lines_with_frame_control(0, (HEIGHT / 2 - 1) as u16, FrameControl::new_first()).await.unwrap();
    fb_display.flush_lines_with_frame_control((HEIGHT / 2) as u16, (HEIGHT - 1) as u16, FrameControl::new_last()).await.unwrap();

    info!("Done!");

    loop {
        embassy_time::Timer::after_secs(1).await;
    }
}

fn draw_text<D>(target: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
{
    const TEXT: &str = "Powered by\ndisplay-driver";
    let shadow_style = MonoTextStyle::new(&FONT_9X18, Rgb565::new(4, 8, 4));
    let text_style = MonoTextStyle::new(&FONT_9X18, Rgb565::WHITE);
    let text_pos = Point::new(120, 200);
    let shadow_offset = Point::new(1, 1);
    let _ = Text::with_alignment(
        TEXT,
        text_pos + shadow_offset,
        shadow_style,
        Alignment::Center,
    )
    .draw(target);
    let _ = Text::with_alignment(TEXT, text_pos, text_style, Alignment::Center).draw(target);
}

fn draw_concentric_gradient<D>(target: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
{
    let center_x: i32 = 120;
    let center_y: i32 = 120;
    let max_radius: f32 = 120.0;
    let center_r: f32 = 255.0;
    let center_g: f32 = 200.0;
    let center_b: f32 = 50.0;
    let edge_r: f32 = 138.0;
    let edge_g: f32 = 43.0;
    let edge_b: f32 = 226.0;

    const BAYER_4X4: [[f32; 4]; 4] = [
        [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
        [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
        [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
        [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
    ];

    for y in 0..240i32 {
        for x in 0..240i32 {
            let dx = (x - center_x) as f32;
            let dy = (y - center_y) as f32;
            let distance = (dx * dx + dy * dy).sqrt();

            let t = if distance >= max_radius {
                1.0
            } else {
                distance / max_radius
            };

            let r_f = lerp(center_r, edge_r, t);
            let g_f = lerp(center_g, edge_g, t);
            let b_f = lerp(center_b, edge_b, t);

            let bayer_threshold = BAYER_4X4[(y & 3) as usize][(x & 3) as usize];

            let dither_r = r_f + (bayer_threshold - 0.5) * 8.226;
            let dither_g = g_f + (bayer_threshold - 0.5) * 4.048;
            let dither_b = b_f + (bayer_threshold - 0.5) * 8.226;

            let r5 = clamp_u8((dither_r / 8.226) as i32, 0, 31) as u8;
            let g6 = clamp_u8((dither_g / 4.048) as i32, 0, 63) as u8;
            let b5 = clamp_u8((dither_b / 8.226) as i32, 0, 31) as u8;

            let color = Rgb565::new(r5, g6, b5);
            let _ = Pixel(Point::new(x, y), color).draw(target);
        }
    }
}

#[inline]
fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

#[inline]
fn clamp_u8(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
