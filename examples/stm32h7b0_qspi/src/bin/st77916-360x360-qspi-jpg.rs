#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ospi::{Config, Ospi};
use embassy_time::Timer;

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{DisplayDriver, FrameControl};
use display_driver_qspi::{QspiConfig, QspiDisplayBus};
use display_driver_st77916::{St77916, NT150XV};

use tjpgd_rs::types::{PixelFormat, Scale};
use tjpgd_rs::JpegDecoder;

use stm32h7b0_qspi_examples::qspi::Stm32OspiDevice;

use embassy_stm32::bind_interrupts;

bind_interrupts!(struct Irqs {
    MDMA => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::MDMA_CH0>;
});

const WIDTH: usize = 360;
const HEIGHT: usize = 360;

// Global framebuffer to avoid stack overflow. 360x360 RGB565 is 259.2 KB.
// Placed in static memory, which goes to .bss section and is zero-initialized.
// Must be aligned to 32 bytes for Cortex-M7 D-Cache cacheline operations.
#[repr(C, align(32))]
struct AlignedBuffer(pub [u8; WIDTH * HEIGHT * 2]);
static mut FB: AlignedBuffer = AlignedBuffer([0; WIDTH * HEIGHT * 2]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START ST77916 QSPI JPG DEMO (Async MDMA)");

    // RCC config
    let config = stm32h7b0_qspi_examples::configure_rcc();

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    // Wiring (STM32H7B0 -> ST77916 360x360 display)
    // PA3 -> CLK
    // PA1 -> IO3
    // PA7 -> IO2
    // PB0 -> IO1
    // PB1 -> IO0
    // PB10 -> CS
    // PA0 -> RST

    let rst = Output::new(p.PA0, Level::High, Speed::High);

    // OSPI Config
    let mut ospi_config = Config::default();
    ospi_config.clock_prescaler = 6; // 280MHz / 7 = 40MHz
    ospi_config.memory_type = embassy_stm32::ospi::MemoryType::Standard;
    ospi_config.device_size = embassy_stm32::ospi::MemorySize::_4GiB; // 4GB to prevent TEF

    let ospi = Ospi::new_quadspi(
        p.OCTOSPI1,
        p.PA3, // CLK
        p.PB1, // IO0
        p.PB0, // IO1
        p.PA7, // IO2
        p.PA1, // IO3
        p.PB10, // CS
        p.MDMA_CH0,
        Irqs,
        ospi_config,
    );

    let device = Stm32OspiDevice { ospi };

    // Create the QSPI Bus
    let qspi_bus_config = QspiConfig::default();
    // Usually ST77916 uses QSPI 1-1-4 or 1-4-4 mode depending on initialization.
    // DisplayDriver QspiDisplayBus manages the line modes for specific phases via QspiConfig if needed.
    let bus = QspiDisplayBus::new(device, qspi_bus_config);

    // Create the Panel
    let panel = St77916::<NT150XV, _, _>::new(LCDResetOption::new_pin(rst));

    // Create and initialize the Driver using builder
    info!("Initializing display...");
    let mut disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();

    info!("Display initialized.");

    let image_data = include_bytes!("../../../assets/cat_360x360.jpg");
    let mut reader = &image_data[..];
    let mut pool = [0u8; 3500]; // TJpgD workspace

    let mut decoder = JpegDecoder::new(&mut pool[..], &mut reader).unwrap();

    info!("Decoding JPEG... {}x{}", decoder.width(), decoder.height());

    // Use core::ptr::addr_of_mut! to avoid warning about mutable reference to static mut
    let fb = unsafe { &mut *core::ptr::addr_of_mut!(FB.0) };

    // Enable caches strictly for the heavy JPEG decoding process!
    // I-Cache and D-Cache give a ~4x speedup on Cortex-M7.
    stm32h7b0_qspi_examples::enable_cache();

    let start_time = embassy_time::Instant::now();
    decoder
        .decode_to_framebuffer(
            fb,
            WIDTH as u16,
            HEIGHT as u16,
            0,
            0,
            PixelFormat::RGB565,
            Scale::None,
        )
        .unwrap();
    let elapsed = start_time.elapsed();
    info!("JPEG Decoded in {} ms!", elapsed.as_millis());

    // D-Cache must be disabled before we perform DMA transfers,
    // because display-driver uses stack-allocated buffers for commands.
    // disable_dcache() automatically cleans (flushes) all dirty cache lines back to SRAM first!
    stm32h7b0_qspi_examples::disable_cache();

    // Send to display in multiple chunks due to STM32 DMA limit (0xFFFF bytes)
    // 360 * 360 * 2 = 259200 bytes.
    // Let's divide into 9 chunks of 40 lines (40 * 360 * 2 = 28800 bytes).
    let chunk_lines = 40;
    let chunk_bytes = chunk_lines * WIDTH * 2;

    use display_driver::Area;

    for chunk in 0..9 {
        let start_line = chunk * chunk_lines;
        let end_line = start_line + chunk_lines - 1;
        let frame_ctrl = FrameControl {
            first: chunk == 0,
            last: chunk == 8,
        };

        disp.write_pixels(
            Area::new(0, start_line as u16, WIDTH as u16, end_line as u16),
            frame_ctrl,
            &fb[chunk * chunk_bytes..(chunk + 1) * chunk_bytes],
        )
        .await
        .unwrap();
    }

    info!("Done!");

    loop {
        Timer::after_secs(1).await;
    }
}
