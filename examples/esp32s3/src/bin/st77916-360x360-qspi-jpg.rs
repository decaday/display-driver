//! ESP32-S3 QSPI ST77916 AMOLED Display JPG Example
//!
//! Drives a ST77916 (NT150XV) AMOLED display over hardware QSPI in 1-1-4 mode.
//! Decodes and displays a 360x360 JPEG image.
//!
//! Panel: NT150XV (360x360).

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

use display_driver::{
    eg::FrameBufferedDisplayDriver, panel::reset::LCDResetOption, ColorFormat, DisplayDriver, FrameControl,
};
use display_driver_st77916::{NT150XV, St77916};
use display_driver_qspi::{QspiConfig, QspiDisplayBus};
use display_driver_esp32s3_examples::qspi::EspHalQspiDevice;

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    pixelcolor::{raw::{BigEndian, RawU16}, Rgb565},
};

use tjpgd_rs::JpegDecoder;
use tjpgd_rs::types::{PixelFormat, Scale};

// ---------------------------------------------------------------------------
// Display geometry
// ---------------------------------------------------------------------------

const WIDTH: usize = 360;
const HEIGHT: usize = 360;

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
    println!("=== ST77916 QSPI Display JPG Example (ESP32-S3) ===");

    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize embassy via esp-rtos
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    // ── Pin assignments ────────────────────────────────────────────────────
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
    
    static TX_DESCRIPTORS: static_cell::StaticCell<[esp_hal::dma::DmaDescriptor; 8]> = static_cell::StaticCell::new();
    let tx_descriptors = TX_DESCRIPTORS.init([esp_hal::dma::DmaDescriptor::EMPTY; 8]);
    
    static BOUNCE_BUF: static_cell::StaticCell<[u8; 256]> = static_cell::StaticCell::new();
    let bounce_buf = BOUNCE_BUF.init([0; 256]);

    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_mhz(10)) 
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

    let bus = QspiDisplayBus::new(device, QspiConfig::default());
    let rst_pin = Output::new(rst, Level::High, OutputConfig::default());
    let panel = St77916::<NT150XV, _, _>::new(LCDResetOption::new_pin(rst_pin));

    // ── Framebuffer ────────────────────────────────────────────────────────
    static mut FB_DATA: core::mem::MaybeUninit<FbType> = core::mem::MaybeUninit::uninit();
    let fb = unsafe {
        let ptr = core::ptr::addr_of_mut!(FB_DATA) as *mut FbType;
        core::ptr::write_bytes(ptr, 0, 1);
        &mut *ptr
    };

    // ── Run async display init & drawing ───────────────────────────────────
    let disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();
    println!("Display initialised.");

    let mut fb_disp = FrameBufferedDisplayDriver::new(disp, fb);
    fb_disp.set_brightness(200).await.unwrap();
    println!("Brightness set to 200.");

    // Decode JPG
    let image_data = include_bytes!("../../../assets/cat_360x360.jpg");
    let mut reader = &image_data[..];
    let mut pool = [0u8; 3500];

    let mut decoder = JpegDecoder::new(&mut pool[..], &mut reader).unwrap();
    println!("Decoding JPEG... {}x{}", decoder.width(), decoder.height());

    let fb_data = unsafe {
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(FB_DATA) as *mut u8,
            WIDTH * HEIGHT * 2
        )
    };

    let start_time = embassy_time::Instant::now();
    decoder.decode_to_framebuffer(
        fb_data,
        WIDTH as u16, 
        HEIGHT as u16, 
        0,
        0,
        PixelFormat::RGB565,
        Scale::None,
    ).unwrap();
    let end_time = embassy_time::Instant::now();
    let elapsed = (end_time - start_time).as_millis();
    println!("JPEG Decoded in {} ms!", elapsed);

    println!("Flushing full framebuffer to display in chunks...");
    
    for chunk in 0..9 {
        let start_line = chunk * 40;
        let end_line = start_line + 39;
        
        let frame_ctrl = FrameControl {
            first: chunk == 0,
            last: chunk == 8,
        };
        
        fb_disp.flush_lines_with_frame_control(start_line as u16, end_line as u16, frame_ctrl)
            .await
            .unwrap();
    }
    
    println!("Done! Display should show the JPEG image.");

    println!("Entering idle loop.");
    let mut ticks = 0;
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1000)).await;
        ticks += 1;
        println!("Tick {}...", ticks);
    }
}
