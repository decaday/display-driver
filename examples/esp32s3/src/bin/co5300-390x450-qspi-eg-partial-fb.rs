//! ESP32-S3 QSPI CO5300 AMOLED Display Example (Partial Framebuffer)
//!
//! This example demonstrates how to drive a 390x450 AMOLED display using a partial framebuffer 
//! (Banding) within internal SRAM, avoiding PSRAM and DMA cache coherency issues.
//!
//! # Performance Note (Partial Buffer Limitations)
//! This is a basic demonstration of the partial framebuffer technique. Because we re-draw the
//! ENTIRE scene for every band (15 times per frame) and wait for the DMA to flush sequentially,
//! it is relatively slow. 
//! 
//! For production usage, you should consider the optimization techniques like dirty rectangles, 
//! ping-pong buffering etc.
//!
//! The SPI frequency is conservatively set to 10 MHz here because breadboards and Dupont wires 
//! often suffer from signal integrity issues at high frequencies, leading to screen tearing.
//! 
//! To achieve a stable 30Hz refresh rate (33.3ms per frame), you indeed need at least ~50MHz.
//! 
//! Note: Due to the AMOLED pixel arrangement, the drawing area's rows and columns must 
//! be even numbers. See https://github.com/decaday/display-driver/tree/master/panels/co5300

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    dma::DmaRxBuf,
    dma_buffers,
    gpio::{Level, Output, OutputConfig},
    interrupt::software::SoftwareInterruptControl,
    spi::{
        master::{Config, Spi},
        Mode,
    },
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;

use embedded_graphics::{
    draw_target::{DrawTarget, DrawTargetExt},
    framebuffer::{buffer_size, Framebuffer},
    geometry::{Point, Size},
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, LittleEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};

use display_driver::{
    eg::FrameBufferedDisplayDriver, panel::reset::LCDResetOption, Area, ColorFormat, DisplayDriver,
    FrameControl,
};
use display_driver_co5300::{spec::Amoled_185Inch_390x450, Co5300};
use display_driver_qspi::{QspiConfig, QspiDisplayBus};
use display_driver_esp32s3_examples::qspi::EspHalQspiDevice;

// ---------------------------------------------------------------------------
// Display geometry
// ---------------------------------------------------------------------------

const WIDTH: usize = 390;
const HEIGHT: usize = 30;

type FbType = Framebuffer<
    Rgb565,
    RawU16,
    BigEndian,
    WIDTH,
    HEIGHT,
    { buffer_size::<Rgb565>(WIDTH, HEIGHT) },
>;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

esp_bootloader_esp_idf::esp_app_desc!();

fn draw_scene<D>(display: &mut D)
where
    D: DrawTarget<Color = Rgb565>,
{
    // 1. Background Grid (subtle lines)
    let grid_style = PrimitiveStyle::with_stroke(Rgb565::new(8, 16, 8), 1);
    for x in (0..390).step_by(30) {
        Line::new(Point::new(x, 0), Point::new(x, 450))
            .into_styled(grid_style)
            .draw(display)
            .ok();
    }
    for y in (0..450).step_by(30) {
        Line::new(Point::new(0, y), Point::new(390, y))
            .into_styled(grid_style)
            .draw(display)
            .ok();
    }

    // 2. Header Bar
    Rectangle::new(Point::new(0, 0), Size::new(390, 40))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::new(0, 10, 31)))
        .draw(display)
        .ok();
        
    let title_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    Text::with_alignment("CO5300 390x450 Dashboard", Point::new(195, 27), title_style, Alignment::Center)
        .draw(display)
        .ok();

    // 3. Status Indicators (Circles)
    let status_style_ok = PrimitiveStyle::with_fill(Rgb565::GREEN);
    let status_style_warn = PrimitiveStyle::with_fill(Rgb565::YELLOW);
    Circle::new(Point::new(20, 10), 16).into_styled(status_style_ok).draw(display).ok();
    Circle::new(Point::new(350, 10), 16).into_styled(status_style_warn).draw(display).ok();

    // 5. Data Chart (Mockup at the bottom)
    let chart_box = Rectangle::new(Point::new(20, 280), Size::new(350, 120));
    chart_box.into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1)).draw(display).ok();
    
    // Draw some random-looking bars
    let bar_style = PrimitiveStyle::with_fill(Rgb565::BLUE);
    let bar_style_alt = PrimitiveStyle::with_fill(Rgb565::new(0, 31, 31)); // Cyan-ish
    for (i, h) in [40, 80, 50, 100, 20, 70, 110, 30].iter().enumerate() {
        let style = if i % 2 == 0 { bar_style } else { bar_style_alt };
        Rectangle::new(
            Point::new(30 + (i as i32 * 40), 400 - *h),
            Size::new(25, *h as u32),
        )
        .into_styled(style)
        .draw(display)
        .ok();
    }

    // 6. Draw Ferris (Center overlap)
    let image_raw: ImageRaw<Rgb565, LittleEndian> = ImageRaw::new(
        include_bytes!("../../../assets/ferris.raw"),
        86, // IMAGE_WIDTH
    );
    // Draw Ferris with a nice white border rectangle
    Rectangle::new(Point::new(147, 137), Size::new(96, 74))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)
        .ok();
    Image::new(&image_raw, Point::new(152, 142))
        .draw(display)
        .ok();

    // 7. Footer
    let footer_style = MonoTextStyle::new(&FONT_10X20, Rgb565::new(16, 32, 16));
    Text::with_alignment("Powered by display-driver", Point::new(195, 435), footer_style, Alignment::Center)
        .draw(display)
        .ok();
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    println!("=== CO5300 QSPI Display Example (ESP32-S3) ===");

    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize embassy via esp-rtos
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    // ── Pin assignments ────────────────────────────────────────────────────
    //   CLK  D0  D1  D2  D3  RST  CS
    let sclk = peripherals.GPIO14;
    let sio0 = peripherals.GPIO13; // D0
    let sio1 = peripherals.GPIO12; // D1
    let sio2 = peripherals.GPIO11; // D2
    let sio3 = peripherals.GPIO10; // D3
    let rst = peripherals.GPIO9;
    let cs = peripherals.GPIO8;

    // ── SPI + DMA (QSPI 1-1-4) ────────────────────────────────────────────
    println!("Configuring SPI2 + DMA (QSPI 1-1-4 mode, 10 MHz)...");

    let (rx_buffer, rx_descriptors, _, _) = dma_buffers!(256, 0);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    
    // Allocate TX descriptors manually (8 descriptors can handle up to 8 * 4092 = 32736 bytes).
    // Our largest chunk is 28800 bytes (60 lines).
    static TX_DESCRIPTORS: static_cell::StaticCell<[esp_hal::dma::DmaDescriptor; 8]> = static_cell::StaticCell::new();
    let tx_descriptors = TX_DESCRIPTORS.init([esp_hal::dma::DmaDescriptor::EMPTY; 8]);
    
    // Bounce buffer for Flash-based initialization commands
    static BOUNCE_BUF: static_cell::StaticCell<[u8; 256]> = static_cell::StaticCell::new();
    let bounce_buf = BOUNCE_BUF.init([0; 256]);

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(10)) // co5300 50mhz max
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_sio0(sio0)
    .with_sio1(sio1)
    .with_sio2(sio2)
    .with_sio3(sio3)
    .with_cs(cs)
    .with_dma(peripherals.DMA_CH0)
    .into_async();

    // ── Display bus & panel ────────────────────────────────────────────────
    let device = EspHalQspiDevice {
        spi: Some(spi),
        rx_buf: Some(dma_rx_buf),
        tx_descriptors: Some(tx_descriptors),
        bounce_buf: Some(bounce_buf),
    };

    // Default QspiConfig: cmd_data_mode=Single, ram_data_mode=Quad (1-1-4)
    let bus = QspiDisplayBus::new(device, QspiConfig::default());

    let rst_pin = Output::new(rst, Level::High, OutputConfig::default());
    let panel = Co5300::<Amoled_185Inch_390x450, _, _>::new(LCDResetOption::new_pin(rst_pin));

    // ── Framebuffer (static, 240×240 RGB565 = 115 200 bytes) ──────────────
    static mut FB_DATA: core::mem::MaybeUninit<FbType> = core::mem::MaybeUninit::uninit();
    let fb = unsafe {
        let ptr = core::ptr::addr_of_mut!(FB_DATA) as *mut FbType;
        // Zero-initialize directly in memory to avoid stack copy
        core::ptr::write_bytes(ptr, 0, 1);
        &mut *ptr
    };

    // ── Run async display init & drawing ───────────────────────────────────
    // Initialise display driver
    let disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();
    println!("Display initialised.");

    // Wrap with framebuffer for embedded-graphics drawing
    let mut fb_disp = FrameBufferedDisplayDriver::new_partial(disp, Area::new(0, 0, WIDTH as u16, HEIGHT as u16), fb).unwrap();

    // Set brightness (0–255)
    fb_disp.set_brightness(200).await.unwrap();
    println!("Brightness set to 200.");

    // ── Draw demo scene ──────────────────────────────────────────
    println!("Flushing framebuffer to display (banding)...");
    let total_height = 450;
    
    for y in (0..total_height).step_by(HEIGHT) {
        // Set display area mapping for this band
        fb_disp.set_area(Area::new(0, y as u16, WIDTH as u16, HEIGHT as u16)).unwrap();
        
        // Clear the local buffer
        fb_disp.clear(Rgb565::BLACK).unwrap();
        
        // Shift drawing origin and draw scene
        let mut translated_disp = fb_disp.translated(Point::new(0, -(y as i32)));
        draw_scene(&mut translated_disp);
        
        // 4. Flush to display with FrameControl
        let frame_ctrl = FrameControl {
            first: y == 0,
            last: y + HEIGHT >= total_height,
        };
        
        fb_disp.flush_with_frame_control(frame_ctrl).await.unwrap();
    }
    
    println!("Done! Display should show the demo scene.");

    println!("Entering idle loop.");
    let mut ticks = 0;
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1000)).await;
        ticks += 1;
        println!("Tick {}...", ticks);
    }
}
