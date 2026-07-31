//! ESP32-S3 QSPI CO5300 AMOLED Display Example
//!
//! Drives a CO5300 AMOLED display over hardware QSPI in 1-1-4 mode
//! (instruction/address on 1 line, pixel data on 4 lines).
//!
//! Panel: GenericCo5300 (240×240) with full-screen framebuffer.

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
    framebuffer::{buffer_size, Framebuffer},
    geometry::{Point, Size},
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle},
    pixelcolor::{
        raw::{BigEndian, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};

use display_driver::{
    eg::FrameBufferedDisplayDriver, panel::reset::LCDResetOption, ColorFormat, DisplayDriver,
    FrameControl,
};
use display_driver_co5300::{spec::GenericCo5300, Co5300};
use display_driver_qspi::{QspiConfig, QspiDisplayBus};
use display_driver_esp32s3_examples::qspi::EspHalQspiDevice;

// ---------------------------------------------------------------------------
// Display geometry
// ---------------------------------------------------------------------------

const WIDTH: usize = 240;
const HEIGHT: usize = 240;

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

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    println!("=== CO5300 QSPI Display Example (ESP32-S3) ===");

    let peripherals = esp_hal::init(esp_hal::Config::default());

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
    let panel = Co5300::<GenericCo5300, _, _>::new(LCDResetOption::new_pin(rst_pin));

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
    let mut fb_disp = FrameBufferedDisplayDriver::new(disp, fb);

    // Set brightness (0–255)
    fb_disp.set_brightness(200).await.unwrap();
    println!("Brightness set to 200.");

    // ── Draw demo scene ──────────────────────────────────────────
    fb_disp.clear(Rgb565::BLACK).unwrap();

    // Three overlapping coloured shapes
    Rectangle::new(Point::new(10, 40), Size::new(100, 100))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(&mut fb_disp)
        .unwrap();

    Rectangle::new(Point::new(70, 70), Size::new(100, 100))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(&mut fb_disp)
        .unwrap();

    Circle::new(Point::new(110, 110), 110)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
        .draw(&mut fb_disp)
        .unwrap();

    // Title text
    let title = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);
    Text::with_alignment("CO5300 QSPI", Point::new(120, 22), title, Alignment::Center)
        .draw(&mut fb_disp)
        .unwrap();

    // Subtitle
    let sub = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::CYAN);
    Text::with_alignment("ESP32-S3 display-driver", Point::new(120, 232), sub, Alignment::Center)
        .draw(&mut fb_disp)
        .unwrap();

    // Flush framebuffer → display
    println!("Flushing framebuffer to display (in chunks)...");
    
    fb_disp.flush_lines_with_frame_control(0, 59, FrameControl::new_first()).await.unwrap();
    fb_disp.flush_lines_with_frame_control(60, 119, FrameControl { first: false, last: false }).await.unwrap();
    fb_disp.flush_lines_with_frame_control(120, 179, FrameControl { first: false, last: false }).await.unwrap();
    fb_disp.flush_lines_with_frame_control(180, 239, FrameControl::new_last()).await.unwrap();
    
    println!("Done! Display should show the demo scene.");

    println!("Entering idle loop.");
    let mut ticks = 0;
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1000)).await;
        ticks += 1;
        println!("Tick {}...", ticks);
    }
}
