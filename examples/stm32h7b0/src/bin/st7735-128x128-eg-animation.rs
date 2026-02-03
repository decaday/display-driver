#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_time::{Instant, Timer};

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Triangle},
    text::{Alignment, Text},
};

use micromath::F32Ext;

use dd_st7735::{spec::vendor_specs::P144H008_V2, spec::MipidcsSpec, St7735};
use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{DisplayDriver, Orientation};
use display_driver_spi::SpiDisplayBus;
use static_cell::StaticCell;

// Native dimensions (128x128 square display)
const WIDTH: usize = P144H008_V2::PHYSICAL_WIDTH as usize;
const HEIGHT: usize = P144H008_V2::PHYSICAL_HEIGHT as usize;

type FramebufferType =
    Framebuffer<Rgb565, RawU16, BigEndian, WIDTH, HEIGHT, { buffer_size::<Rgb565>(WIDTH, HEIGHT) }>;

static FB: StaticCell<FramebufferType> = StaticCell::new();

/// Draw a creative animated scene with geometric patterns
fn draw_creative_scene(fb: &mut FramebufferType, frame: u32) {
    fb.clear(Rgb565::BLACK).unwrap();

    let center_x = (WIDTH / 2) as i32;
    let center_y = (HEIGHT / 2) as i32;

    // Animated rotating triangles
    let angle = (frame % 360) as f32 * 3.14159 / 180.0;
    let radius = 40;

    for i in 0..3 {
        let offset_angle = angle + (i as f32 * 2.0 * 3.14159 / 3.0);
        let x = center_x + (radius as f32 * offset_angle.cos()) as i32;
        let y = center_y + (radius as f32 * offset_angle.sin()) as i32;

        let color = match i {
            0 => Rgb565::RED,
            1 => Rgb565::GREEN,
            _ => Rgb565::BLUE,
        };

        Circle::new(Point::new(x - 8, y - 8), 16)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(fb)
            .ok();
    }

    // Pulsating center circle
    let pulse = ((frame % 60) as f32 / 60.0 * 2.0 * 3.14159).sin();
    let pulse_radius = (15.0 + pulse * 5.0) as u32;

    Circle::new(
        Point::new(
            center_x - pulse_radius as i32,
            center_y - pulse_radius as i32,
        ),
        pulse_radius * 2,
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::CYAN, 2))
    .draw(fb)
    .ok();

    // Animated corner triangles
    let corner_offset = ((frame / 2) % 20) as i32;

    // Top-left
    Triangle::new(
        Point::new(0, 0),
        Point::new(20 + corner_offset, 0),
        Point::new(0, 20 + corner_offset),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::MAGENTA))
    .draw(fb)
    .ok();

    // Top-right
    Triangle::new(
        Point::new(WIDTH as i32 - 1, 0),
        Point::new(WIDTH as i32 - 1 - (20 + corner_offset), 0),
        Point::new(WIDTH as i32 - 1, 20 + corner_offset),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::YELLOW))
    .draw(fb)
    .ok();

    // Bottom-left
    Triangle::new(
        Point::new(0, HEIGHT as i32 - 1),
        Point::new(20 + corner_offset, HEIGHT as i32 - 1),
        Point::new(0, HEIGHT as i32 - 1 - (20 + corner_offset)),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::YELLOW))
    .draw(fb)
    .ok();

    // Bottom-right
    Triangle::new(
        Point::new(WIDTH as i32 - 1, HEIGHT as i32 - 1),
        Point::new(WIDTH as i32 - 1 - (20 + corner_offset), HEIGHT as i32 - 1),
        Point::new(WIDTH as i32 - 1, HEIGHT as i32 - 1 - (20 + corner_offset)),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::MAGENTA))
    .draw(fb)
    .ok();

    // Animated grid lines
    let grid_offset = ((frame / 3) % 16) as i32;

    for i in 0..8 {
        let pos = i * 16 + grid_offset;

        // Vertical lines
        if pos < WIDTH as i32 {
            Line::new(Point::new(pos, 0), Point::new(pos, HEIGHT as i32 - 1))
                .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(0, 8, 8), 1))
                .draw(fb)
                .ok();
        }

        // Horizontal lines
        if pos < HEIGHT as i32 {
            Line::new(Point::new(0, pos), Point::new(WIDTH as i32 - 1, pos))
                .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(0, 8, 8), 1))
                .draw(fb)
                .ok();
        }
    }

    // Title text with shadow effect
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let shadow_style = MonoTextStyle::new(&FONT_6X10, Rgb565::new(8, 8, 8));

    let title_text = "ANIMATION DEMO";
    // Shadow
    Text::with_alignment(
        title_text,
        Point::new(center_x + 1, 11),
        shadow_style,
        Alignment::Center,
    )
    .draw(fb)
    .ok();

    // Main text
    Text::with_alignment(
        title_text,
        Point::new(center_x, 10),
        text_style,
        Alignment::Center,
    )
    .draw(fb)
    .ok();

    // Bottom text
    Text::with_alignment(
        "display-driver",
        Point::new(center_x, HEIGHT as i32 - 5),
        MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
        Alignment::Center,
    )
    .draw(fb)
    .ok();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START EG ANIMATION DEMO - 128x128");

    // RCC config
    let config = stm32h7b0_examples::configure_rcc();

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    let dc = Output::new(p.PE13, Level::Low, Speed::High);
    let cs = Output::new(p.PE11, Level::High, Speed::High);
    let _lcd_led = Output::new(p.PE10, Level::Low, Speed::Low);
    let rst = Output::new(p.PE15, Level::High, Speed::Low);

    let mut spi_config: spi::Config = Default::default();
    spi_config.frequency = Hertz(24_000_000);

    let spi = Spi::new_txonly(p.SPI4, p.PE12, p.PE14, p.DMA1_CH0, spi_config);

    // Create the SPI Bus
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let bus = SpiDisplayBus::new(spi_device, dc);

    // Create the Panel
    let panel = St7735::<P144H008_V2, _, _>::new(LCDResetOption::new_pin(rst));

    // Create and initialize the Driver using builder
    info!("Initializing display...");
    let mut disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .with_orientation(Orientation::Deg0)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();
    info!("Display initialized.");

    // Initialize global framebuffer
    let fb = FB.init(Framebuffer::new());

    // Animation loop with FPS control
    let mut frame: u32 = 0;
    let target_fps = 30;
    let target_frame_time_ms = 1000 / target_fps; // 33ms for 30 FPS

    // Measure first frame time
    info!("Measuring first frame rendering time...");
    let start = Instant::now();

    draw_creative_scene(fb, 0);
    disp.write_frame(fb.data()).await.unwrap();

    let first_frame_duration = start.elapsed();
    let first_frame_ms = first_frame_duration.as_millis();

    // Check if we can achieve target FPS
    if first_frame_ms > target_frame_time_ms as u64 {
        defmt::warn!(
            "⚠️  Cannot achieve {} FPS! Frame rendering takes {} ms, but target is {} ms",
            target_fps,
            first_frame_ms,
            target_frame_time_ms
        );
        defmt::warn!(
            "Maximum achievable FPS: ~{}",
            if first_frame_ms > 0 {
                1000 / first_frame_ms
            } else {
                0
            }
        );
    } else {
        info!(
            "✓ Can achieve {} FPS (frame: {} ms, target: {} ms)",
            target_fps, first_frame_ms, target_frame_time_ms
        );
    }

    loop {
        let frame_start = Instant::now();

        draw_creative_scene(fb, frame);
        disp.write_frame(fb.data()).await.unwrap();

        frame = frame.wrapping_add(1);

        // Calculate precise delay to maintain target FPS
        let elapsed = frame_start.elapsed().as_millis();

        if elapsed < target_frame_time_ms as u64 {
            let delay_ms = target_frame_time_ms as u64 - elapsed;
            Timer::after_millis(delay_ms).await;
        }
        // If frame took longer than target, continue immediately without delay
    }
}
