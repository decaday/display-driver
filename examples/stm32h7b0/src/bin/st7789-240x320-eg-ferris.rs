#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, LittleEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{Area, DisplayDriver, FrameControl, Orientation};
use display_driver_spi::SpiDisplayBus;
use display_driver_st7789::{spec::generic::Generic240x320Type2, spec::PanelSpec, St7789};


const IMAGE_WIDTH: usize = 86;
#[allow(unused)]
const IMAGE_HEIGHT: usize = 64;

// Rotation 90
const SCREEN_WIDTH: usize = Generic240x320Type2::PHYSICAL_HEIGHT as _;
const SCREEN_HEIGHT: usize = Generic240x320Type2::PHYSICAL_WIDTH as _;

type FramebufferType = Framebuffer<
    Rgb565,
    RawU16,
    BigEndian,
    SCREEN_WIDTH,
    SCREEN_HEIGHT,
    { buffer_size::<Rgb565>(SCREEN_WIDTH, SCREEN_HEIGHT) },
>;

static mut FB: FramebufferType = Framebuffer::new();

embassy_stm32::bind_interrupts!(struct Irqs {
    DMA1_STREAM0 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START");

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

    let spi = Spi::new_txonly(p.SPI4, p.PE12, p.PE14, p.DMA1_CH0, Irqs, spi_config);

    // Create the SPI Bus
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let bus = SpiDisplayBus::new(spi_device, dc);

    // Create the Panel
    let panel = St7789::<Generic240x320Type2, _, _>::new(LCDResetOption::new_pin(rst));

    // Create and initialize the Driver using builder
    info!("Initializing display...");
    let mut disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .with_orientation(Orientation::Deg90)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();

    info!("Display initialized.");

    let fb = unsafe { &mut *core::ptr::addr_of_mut!(FB) };

    fb.clear(Rgb565::BLACK).unwrap();

    // Draw background grid
    let grid_style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::new(4, 8, 4))
        .stroke_width(1)
        .build();

    for x in (0..SCREEN_WIDTH).step_by(20) {
        Line::new(Point::new(x as i32, 0), Point::new(x as i32, SCREEN_HEIGHT as i32))
            .into_styled(grid_style)
            .draw(fb)
            .unwrap();
    }
    for y in (0..SCREEN_HEIGHT).step_by(20) {
        Line::new(Point::new(0, y as i32), Point::new(SCREEN_WIDTH as i32, y as i32))
            .into_styled(grid_style)
            .draw(fb)
            .unwrap();
    }

    // Draw L-shaped markers at the corners to verify offsets
    // Drawn after the grid so they are on top!
    stm32h7b0_examples::LShapedMarkers::new(
        SCREEN_WIDTH as i32,
        SCREEN_HEIGHT as i32,
        10,
        Rgb565::RED,
    )
    .draw(fb)
    .unwrap();

    // Draw a big circle in the middle
    Circle::with_center(Point::new(SCREEN_WIDTH as i32 / 2, SCREEN_HEIGHT as i32 / 2), 160)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::BLUE, 3))
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
            y: (SCREEN_HEIGHT - IMAGE_HEIGHT) as i32 / 2 - 20,
        },
    );

    image.draw(fb).unwrap();

    // Draw Text
    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    Text::new("powered by display-driver", Point::new(35, 180), style)
        .draw(fb)
        .unwrap();
        
    let style_small = MonoTextStyle::new(&embedded_graphics::mono_font::ascii::FONT_6X10, Rgb565::CYAN);
    Text::new("ST7789 320x240 Demo", Point::new(100, 210), style_small)
        .draw(fb)
        .unwrap();

    // Helper macro to flush framebuffer in chunks
    macro_rules! flush_fb {
        () => {
            let chunk_lines = 80;
            let chunk_bytes = chunk_lines * 320 * 2;
            
            disp.write_pixels(
                Area::from_origin(320, chunk_lines as u16),
                FrameControl::new_first(),
                &fb.data()[0..chunk_bytes],
            ).await.unwrap();

            disp.write_pixels(
                Area::new(0, chunk_lines as u16, 320, chunk_lines as u16),
                FrameControl { first: false, last: false },
                &fb.data()[chunk_bytes..chunk_bytes * 2],
            ).await.unwrap();

            disp.write_pixels(
                Area::new(0, (chunk_lines * 2) as u16, 320, chunk_lines as u16),
                FrameControl::new_last(),
                &fb.data()[chunk_bytes * 2..chunk_bytes * 3],
            ).await.unwrap();
        }
    }

    // Flush to display
    info!("Flushing to display...");
    flush_fb!();
    info!("Drawing finished.");
    
    // Draw Progress Bar Track
    let bar_width = 280;
    let bar_height = 12;
    let bar_x = (SCREEN_WIDTH as i32 - bar_width) / 2;
    let bar_y = 218;
    
    Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_width as u32, bar_height as u32))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
        .draw(fb)
        .unwrap();
    
    // Progress Bar Animation Loop
    let mut progress: i32 = 0;
    
    loop {
        // Clear the inner progress area
        Rectangle::new(
            Point::new(bar_x + 2, bar_y + 2), 
            Size::new((bar_width - 4) as u32, (bar_height - 4) as u32)
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(fb)
        .unwrap();
            
        // Draw the current progress
        if progress > 0 {
            let current_width = ((bar_width - 4) * progress) / 100;
            Rectangle::new(
                Point::new(bar_x + 2, bar_y + 2), 
                Size::new(current_width as u32, (bar_height - 4) as u32)
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
            .draw(fb)
            .unwrap();
        }
            
        flush_fb!();
        
        progress += 2;
        if progress > 100 {
            // Pause shortly at 100% before restarting
            embassy_time::Timer::after_millis(500).await;
            progress = 0;
        } else {
            embassy_time::Timer::after_millis(30).await;
        }
    }
}
