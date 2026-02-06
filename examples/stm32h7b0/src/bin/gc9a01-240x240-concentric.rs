#![no_main]
#![no_std]

//! GC9A01 240x240 Concentric Gradient Demo
//!
//! This example demonstrates driving a round GC9A01 display with a
//! 240x240 resolution. It uses a global framebuffer in AXI SRAM to
//! avoid excessive stack usage and draws a concentric gradient pattern.

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    geometry::Point,
    mono_font::{ascii::FONT_9X18, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
    text::{Alignment, Text},
};
use micromath::F32Ext;

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{Area, DisplayDriver, FrameControl, Orientation};
use display_driver_gc9a01::{spec::Generic240x240Type1, Gc9a01};
use display_driver_spi::SpiDisplayBus;
use static_cell::StaticCell;

const WIDTH: usize = 240;
const HEIGHT: usize = 240;

type FramebufferType =
    Framebuffer<Rgb565, RawU16, BigEndian, WIDTH, HEIGHT, { buffer_size::<Rgb565>(WIDTH, HEIGHT) }>;

static FB: StaticCell<FramebufferType> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START GC9A01 CONCENTRIC GRADIENT DEMO");

    // RCC config
    let config = stm32h7b0_examples::configure_rcc();

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    let dc = Output::new(p.PE13, Level::Low, Speed::High);
    let cs = Output::new(p.PE9, Level::High, Speed::High);
    let rst = Output::new(p.PE15, Level::High, Speed::High);
    let _lcd_led = Output::new(p.PE10, Level::Low, Speed::Low);

    let mut spi_config: spi::Config = Default::default();
    spi_config.frequency = Hertz(10_000_000);

    let spi = Spi::new_txonly(p.SPI4, p.PE12, p.PE14, p.DMA1_CH0, spi_config);

    // Create the SPI Bus
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let bus = SpiDisplayBus::new(spi_device, dc);

    // Create the Panel
    let panel = Gc9a01::<Generic240x240Type1, _, _>::new(LCDResetOption::new_pin(rst));

    // Create and initialize the Driver using builder
    info!("Initializing display...");
    let mut disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .with_orientation(Orientation::Deg180)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();

    info!("Display initialized.");

    // Initialize framebuffer
    let fb = FB.init(Framebuffer::new());

    // Draw content
    draw_concentric_gradient(fb);
    draw_text(fb);

    // Flush to display
    info!("Flushing to display...");

    // Split transfer into two chunks because STM32 DMA limit is 65535 bytes
    // Total size: 240 * 240 * 2 = 115200 bytes
    // Half size: 115200 / 2 = 57600 bytes
    let data = fb.data();
    let (first, second) = data.split_at(data.len() / 2);

    // Send first half (Top 240x120)
    disp.write_pixels(
        Area::from_origin(WIDTH as u16, (HEIGHT / 2) as u16),
        FrameControl::new_first(),
        first,
    )
    .await
    .unwrap();

    // Send second half (Bottom 240x120)
    disp.write_pixels(
        Area::new(0, (HEIGHT / 2) as u16, WIDTH as u16, (HEIGHT / 2) as u16),
        FrameControl::new_last(),
        second,
    )
    .await
    .unwrap();

    info!("Done!");

    loop {}
}

/// Draw "Powered by display-driver" text with shadow at bottom center.
fn draw_text<D>(target: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
{
    const TEXT: &str = "Powered by\ndisplay-driver";

    // Shadow style (dark, semi-transparent effect)
    let shadow_style = MonoTextStyle::new(&FONT_9X18, Rgb565::new(4, 8, 4));
    // Main text style (white)
    let text_style = MonoTextStyle::new(&FONT_9X18, Rgb565::WHITE);

    // Center position at bottom (y=200, centered horizontally at x=120)
    let text_pos = Point::new(120, 200);
    let shadow_offset = Point::new(1, 1);

    // Draw shadow first (offset by +1, +1)
    let _ = Text::with_alignment(
        TEXT,
        text_pos + shadow_offset,
        shadow_style,
        Alignment::Center,
    )
    .draw(target);

    // Draw main text on top
    let _ = Text::with_alignment(TEXT, text_pos, text_style, Alignment::Center).draw(target);
}

/// Draw a sunset gradient with ordered dithering to reduce Mach banding.
///
/// Uses per-pixel rendering with 4x4 Bayer matrix dithering to create smooth
/// color transitions despite RGB565's limited color depth.
fn draw_concentric_gradient<D>(target: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
{
    let center_x: i32 = 120;
    let center_y: i32 = 120;
    let max_radius: f32 = 120.0;

    // Sunset gradient colors (in 8-bit RGB for higher precision interpolation)
    // Center: Golden Yellow (255, 200, 50)
    // Edge: Violet (138, 43, 226)
    let center_r: f32 = 255.0;
    let center_g: f32 = 200.0;
    let center_b: f32 = 50.0;

    let edge_r: f32 = 138.0;
    let edge_g: f32 = 43.0;
    let edge_b: f32 = 226.0;

    // 4x4 Bayer ordered dithering matrix (normalized to 0.0-1.0 range)
    // This creates a repeating threshold pattern that distributes quantization
    // error spatially, reducing the perception of color banding.
    const BAYER_4X4: [[f32; 4]; 4] = [
        [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
        [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
        [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
        [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
    ];

    // Render each pixel individually for proper dithering
    for y in 0..240i32 {
        for x in 0..240i32 {
            // Calculate distance from center
            let dx = (x - center_x) as f32;
            let dy = (y - center_y) as f32;
            let distance = (dx * dx + dy * dy).sqrt();

            // Clamp distance to max_radius and calculate interpolation factor
            let t = if distance >= max_radius {
                1.0
            } else {
                distance / max_radius
            };

            // Linear interpolation in 8-bit color space for precision
            let r_f = lerp(center_r, edge_r, t);
            let g_f = lerp(center_g, edge_g, t);
            let b_f = lerp(center_b, edge_b, t);

            // Get Bayer threshold for this pixel position
            let bayer_threshold = BAYER_4X4[(y & 3) as usize][(x & 3) as usize];

            // Apply ordered dithering: add threshold before quantization
            // Scale factor accounts for the bit depth difference (8-bit to 5/6-bit)
            // R/B: 255 -> 31 (8.226 per level), G: 255 -> 63 (4.048 per level)
            let dither_r = r_f + (bayer_threshold - 0.5) * 8.226;
            let dither_g = g_f + (bayer_threshold - 0.5) * 4.048;
            let dither_b = b_f + (bayer_threshold - 0.5) * 8.226;

            // Quantize to RGB565 with clamping
            let r5 = clamp_u8((dither_r / 8.226) as i32, 0, 31) as u8;
            let g6 = clamp_u8((dither_g / 4.048) as i32, 0, 63) as u8;
            let b5 = clamp_u8((dither_b / 8.226) as i32, 0, 31) as u8;

            let color = Rgb565::new(r5, g6, b5);

            // Draw single pixel using a 1x1 circle (more efficient than Rectangle)
            let _ = Circle::with_center(Point::new(x, y), 1)
                .into_styled(PrimitiveStyle::with_fill(color))
                .draw(target);
        }
    }
}

/// Linear interpolation between two values.
#[inline]
fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

/// Clamp an i32 value to a u8 range.
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
