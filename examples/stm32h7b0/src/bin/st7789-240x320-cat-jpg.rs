#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_time::Timer;

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{Area, DisplayDriver, FrameControl, Orientation};
use display_driver_spi::SpiDisplayBus;
use display_driver_st7789::{spec::generic::Generic240x320Type2, spec::PanelSpec, St7789};

use tjpgd_rs::JpegDecoder;
use tjpgd_rs::types::{PixelFormat, Scale};

// Native dimensions (Portrait 0 degree)
const P_WIDTH: usize = Generic240x320Type2::PHYSICAL_WIDTH as usize;
const P_HEIGHT: usize = Generic240x320Type2::PHYSICAL_HEIGHT as usize;

embassy_stm32::bind_interrupts!(struct Irqs {
    DMA1_STREAM0 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH0>;
});

// Global framebuffer to avoid stack overflow. 240x320 RGB565 is 153.6 KB.
// Placed in static memory, which goes to .bss section and is zero-initialized.
// Must be aligned to 32 bytes for Cortex-M7 D-Cache cacheline operations.
#[repr(C, align(32))]
struct AlignedBuffer(pub [u8; P_WIDTH * P_HEIGHT * 2]);
static mut FB: AlignedBuffer = AlignedBuffer([0; P_WIDTH * P_HEIGHT * 2]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START ST7789 CAT JPG DEMO");

    // RCC config
    let config = stm32h7b0_examples::configure_rcc();

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    // Wiring (STM32H7B0 -> ST7789 240x320 display)
    // 3V3  -> VCC     | GND -> GND
    // PE12 -> SCL/CLK | PE14 -> SDA/DIN
    // PE9  -> CS      | PE13 -> DC
    // PE15 -> RST     | PE10 -> BL/LED
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
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();
    
    // The image is 320x240 (landscape), but our display is 240x320 (portrait).
    // Let's set orientation to Deg90 so it fits nicely.
    disp.set_orientation(Orientation::Deg90).await.unwrap();
    info!("Display initialized.");

    let image_data = include_bytes!("../../../assets/cat1_320x240.jpg");
    let mut reader = &image_data[..];
    let mut pool = [0u8; 3500]; // TJpgD workspace

    let mut decoder = JpegDecoder::new(&mut pool[..], &mut reader).unwrap();
    
    info!("Decoding JPEG... {}x{}", decoder.width(), decoder.height());

    // Use core::ptr::addr_of_mut! to avoid warning about mutable reference to static mut
    let fb = unsafe { &mut *core::ptr::addr_of_mut!(FB.0) };
    
    // Our target image is 320x240. The orientation is Deg90 so the logical width is 320.
    let target_width = 320;

    // Enable caches strictly for the heavy JPEG decoding process!
    // I-Cache and D-Cache give a ~4x speedup on Cortex-M7.
    // We will enable caches later, right before decoding the JPEG, 
    // to avoid corrupting SPI DMA transfers during display initialization.
    stm32h7b0_examples::enable_cache();

    let start_time = embassy_time::Instant::now();
    decoder.decode_to_framebuffer(
        fb,
        target_width as u16,
        240, // Height is 240 in this orientation
        0,
        0,
        PixelFormat::RGB565,
        Scale::None,
    ).unwrap();
    let elapsed = start_time.elapsed();
    info!("JPEG Decoded in {} ms!", elapsed.as_millis());

    // D-Cache must be disabled before we perform DMA transfers for SPI,
    // because display-driver uses stack-allocated buffers for commands.
    // If D-Cache is left on, DMA will read stale data from SRAM.
    // disable_dcache() automatically cleans (flushes) all dirty cache lines back to SRAM first!
    stm32h7b0_examples::disable_cache();
    
    // Send to display in multiple chunks due to STM32 DMA limit (0xFFFF bytes)
    // 320 * 240 * 2 = 153600 bytes
    // We send in 3 chunks of 80 lines each (51200 bytes per chunk).
    let chunk_lines = 80;
    let chunk_bytes = chunk_lines * 320 * 2;
    
    disp.write_pixels(
        Area::from_origin(320, chunk_lines as u16),
        FrameControl::new_first(),
        &fb[0..chunk_bytes],
    ).await.unwrap();

    disp.write_pixels(
        Area::new(0, chunk_lines as u16, 320, chunk_lines as u16),
        FrameControl { first: false, last: false },
        &fb[chunk_bytes..chunk_bytes * 2],
    ).await.unwrap();

    disp.write_pixels(
        Area::new(0, (chunk_lines * 2) as u16, 320, chunk_lines as u16),
        FrameControl::new_last(),
        &fb[chunk_bytes * 2..chunk_bytes * 3],
    ).await.unwrap();

    info!("Done!");

    loop {
        Timer::after_secs(1).await;
    }
}
