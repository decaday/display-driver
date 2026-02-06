#![no_main]
#![no_std]

//! Computer Monitor UI Demo for ST7789 135x240 Display
//!
//! This example demonstrates a visually stunning static interface using
//! embedded-graphics.

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    geometry::{Point, Size},
    mono_font::{ascii::FONT_6X10, ascii::FONT_9X15_BOLD, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::{
        Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle,
        StrokeAlignment, Triangle,
    },
    text::Text,
};

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{DisplayDriver, Orientation};
use display_driver_spi::SpiDisplayBus;
use display_driver_st7789::{spec::generic::Generic135x240Type1, spec::PanelSpec, St7789};
use static_cell::StaticCell;

// Portrait mode: 135x240
const SCREEN_WIDTH: usize = Generic135x240Type1::PHYSICAL_WIDTH as usize;
const SCREEN_HEIGHT: usize = Generic135x240Type1::PHYSICAL_HEIGHT as usize;

type FramebufferType = Framebuffer<
    Rgb565,
    RawU16,
    BigEndian,
    SCREEN_WIDTH,
    SCREEN_HEIGHT,
    { buffer_size::<Rgb565>(SCREEN_WIDTH, SCREEN_HEIGHT) },
>;

static FB: StaticCell<FramebufferType> = StaticCell::new();

// Color palette - Modern dark theme with vibrant accents
mod colors {
    use embedded_graphics::pixelcolor::{Rgb565, RgbColor};

    pub const BACKGROUND: Rgb565 = Rgb565::new(1, 2, 4); // Deep dark blue-black
    pub const CARD_BG: Rgb565 = Rgb565::new(3, 6, 10); // Slightly lighter card
    pub const ACCENT_CYAN: Rgb565 = Rgb565::new(0, 28, 31); // Vibrant cyan
    pub const ACCENT_MAGENTA: Rgb565 = Rgb565::new(31, 4, 20); // Hot pink/magenta
    pub const ACCENT_ORANGE: Rgb565 = Rgb565::new(31, 16, 0); // Warm orange
    pub const ACCENT_GREEN: Rgb565 = Rgb565::new(4, 31, 12); // Neon green
    pub const TEXT_PRIMARY: Rgb565 = Rgb565::WHITE;
    pub const TEXT_SECONDARY: Rgb565 = Rgb565::new(20, 20, 22); // Light gray
    pub const BORDER_GLOW: Rgb565 = Rgb565::new(8, 16, 24); // Subtle blue border
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START ST7789 MONITOR UI DEMO");

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
    let panel = St7789::<Generic135x240Type1, _, _>::new(LCDResetOption::new_pin(rst));

    // Create and initialize the Driver using builder
    info!("Initializing display...");
    let mut disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .with_orientation(Orientation::Deg0)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();

    info!("Display initialized.");

    // Initialize framebuffer
    let fb = FB.init(Framebuffer::new());

    draw_ui(fb);

    // Flush to display
    info!("Flushing to display...");
    disp.write_frame(fb.data()).await.unwrap();

    info!("Beautiful UI rendered!");

    loop {}
}

fn draw_ui<D>(fb: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    // Clear with gradient-like background
    fb.clear(colors::BACKGROUND).unwrap();

    // Draw decorative top gradient bar
    draw_gradient_bar(fb, 0, 8, colors::ACCENT_CYAN, colors::ACCENT_MAGENTA);

    // Header section
    draw_header(fb);

    // Status cards
    draw_status_card(fb, 8, 45, 119, 50, "CPU", "78%", colors::ACCENT_CYAN);
    draw_status_card(fb, 8, 100, 119, 50, "MEM", "4.2GB", colors::ACCENT_MAGENTA);

    // Activity indicator with animated-look dots
    draw_activity_section(fb, 155);

    // Bottom decorative elements
    draw_bottom_decoration(fb);
}

fn draw_gradient_bar<D>(fb: &mut D, y: i32, height: i32, color1: Rgb565, color2: Rgb565)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    // Simulate gradient with alternating colored lines
    for i in 0..height {
        let color = if i % 2 == 0 { color1 } else { color2 };
        Line::new(Point::new(0, y + i), Point::new(134, y + i))
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(fb)
            .ok();
    }
}

fn draw_header<D>(fb: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    let title_style = MonoTextStyle::new(&FONT_9X15_BOLD, colors::TEXT_PRIMARY);
    let subtitle_style = MonoTextStyle::new(&FONT_6X10, colors::TEXT_SECONDARY);

    // Title with glow effect (draw slightly offset in accent color first)
    Text::new("SYSTEM", Point::new(10, 28), title_style)
        .draw(fb)
        .ok();

    Text::new(
        "MONITOR",
        Point::new(75, 28),
        MonoTextStyle::new(&FONT_9X15_BOLD, colors::ACCENT_CYAN),
    )
    .draw(fb)
    .ok();

    // Subtitle
    Text::new("Real-time Dashboard", Point::new(10, 40), subtitle_style)
        .draw(fb)
        .ok();
}

fn draw_status_card<D>(
    fb: &mut D,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    label: &str,
    value: &str,
    accent: Rgb565,
) where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    // Card background with rounded corners
    let card_style = PrimitiveStyleBuilder::new()
        .fill_color(colors::CARD_BG)
        .stroke_color(colors::BORDER_GLOW)
        .stroke_width(1)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(x, y), Size::new(w, h)),
        Size::new(6, 6),
    )
    .into_styled(card_style)
    .draw(fb)
    .ok();

    // Accent bar on left side
    Rectangle::new(Point::new(x, y + 2), Size::new(3, h - 4))
        .into_styled(PrimitiveStyle::with_fill(accent))
        .draw(fb)
        .ok();

    // Label
    let label_style = MonoTextStyle::new(&FONT_6X10, colors::TEXT_SECONDARY);
    Text::new(label, Point::new(x + 10, y + 15), label_style)
        .draw(fb)
        .ok();

    // Value - large and prominent
    let value_style = MonoTextStyle::new(&FONT_9X15_BOLD, colors::TEXT_PRIMARY);
    Text::new(value, Point::new(x + 10, y + 35), value_style)
        .draw(fb)
        .ok();

    // Progress bar
    let progress_y = y + (h as i32) - 8;
    let bar_width = (w as i32) - 20;

    // Background bar
    Rectangle::new(
        Point::new(x + 10, progress_y),
        Size::new(bar_width as u32, 4),
    )
    .into_styled(PrimitiveStyle::with_fill(colors::BACKGROUND))
    .draw(fb)
    .ok();

    // Filled progress (simulate based on label)
    let fill_percent = if label == "CPU" { 78 } else { 60 };
    let fill_width = (bar_width * fill_percent / 100) as u32;

    Rectangle::new(Point::new(x + 10, progress_y), Size::new(fill_width, 4))
        .into_styled(PrimitiveStyle::with_fill(accent))
        .draw(fb)
        .ok();
}

fn draw_activity_section<D>(fb: &mut D, y: i32)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    let label_style = MonoTextStyle::new(&FONT_6X10, colors::TEXT_SECONDARY);

    Text::new("ACTIVITY", Point::new(10, y + 12), label_style)
        .draw(fb)
        .ok();

    // Animated-look signal dots
    let dot_colors = [
        colors::ACCENT_GREEN,
        colors::ACCENT_CYAN,
        colors::ACCENT_MAGENTA,
        colors::ACCENT_ORANGE,
    ];

    for (i, &color) in dot_colors.iter().enumerate() {
        let dot_x = 80 + (i as i32 * 14);
        Circle::new(Point::new(dot_x, y + 4), 8)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(fb)
            .ok();
    }

    // Network activity visualization - simple wave pattern
    let wave_y = y + 25;
    for i in 0..12 {
        let x = 10 + i * 10;
        let heights = [6, 12, 8, 15, 10, 18, 7, 14, 9, 16, 11, 8];
        let h = heights[i as usize];

        Rectangle::new(Point::new(x, wave_y + (20 - h)), Size::new(6, h as u32))
            .into_styled(PrimitiveStyle::with_fill(colors::ACCENT_CYAN))
            .draw(fb)
            .ok();
    }
}

fn draw_bottom_decoration<D>(fb: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    // Bottom status bar
    let y = 210;

    // Separator line
    Line::new(Point::new(10, y), Point::new(124, y))
        .into_styled(PrimitiveStyle::with_stroke(colors::BORDER_GLOW, 1))
        .draw(fb)
        .ok();

    // Status icons (circles)
    let status_style = PrimitiveStyle::with_fill(colors::ACCENT_GREEN);
    Circle::new(Point::new(15, y + 8), 6)
        .into_styled(status_style)
        .draw(fb)
        .ok();

    // Status text
    let status_text_style = MonoTextStyle::new(&FONT_6X10, colors::TEXT_SECONDARY);
    Text::new("Online", Point::new(28, y + 15), status_text_style)
        .draw(fb)
        .ok();

    // Decorative triangles
    let tri_style = PrimitiveStyle::with_fill(colors::ACCENT_MAGENTA);
    Triangle::new(
        Point::new(100, y + 18),
        Point::new(108, y + 5),
        Point::new(116, y + 18),
    )
    .into_styled(tri_style)
    .draw(fb)
    .ok();

    // Bottom accent bar
    draw_gradient_bar(fb, 232, 8, colors::ACCENT_MAGENTA, colors::ACCENT_CYAN);
}
