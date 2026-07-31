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
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Level, Output, OutputConfig},
    interrupt::software::SoftwareInterruptControl,
    spi::{
        master::{Address, Command, Config, DataMode, Spi},
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
use display_driver_qspi::{LineMode, PhaseConfig, QspiConfig, QspiDevice, QspiDisplayBus, QspiTransaction};

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
// QspiDevice adapter – bridges display-driver-qspi to esp-hal SpiDma
// ---------------------------------------------------------------------------

/// Wraps an esp-hal `SpiDma<Async>` to implement the [`QspiDevice`] trait
/// required by `QspiDisplayBus`.
struct EspHalQspiDevice<'d> {
    spi: Option<esp_hal::spi::master::SpiDma<'d, esp_hal::Async>>,
    rx_buf: Option<DmaRxBuf>,
    tx_descriptors: Option<&'static mut [esp_hal::dma::DmaDescriptor]>,
    bounce_buf: Option<&'static mut [u8]>,
}

impl<'d> QspiDevice for EspHalQspiDevice<'d> {
    type Error = esp_hal::spi::Error;

    async fn write(
        &mut self,
        transaction: &QspiTransaction,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let cmd = to_esp_command(&transaction.instruction);
        let addr = to_esp_address(&transaction.address);
        let data_mode = to_esp_data_mode(transaction.data_mode);

        let tx_descriptors = self.tx_descriptors.take().unwrap();
        let bounce_buf_full = self.bounce_buf.take().unwrap();
        
        let tx_buf = match DmaOperationKind::for_write(data) {
            DmaOperationKind::InPlace => {
                // Zero-copy path for SRAM data (e.g., Framebuffer).
                let data_ptr = data.as_ptr() as *mut u8;
                let data_static: &'static mut [u8] = unsafe { core::slice::from_raw_parts_mut(data_ptr, data.len()) };
                DmaTxBuf::new(tx_descriptors, data_static).unwrap()
            }
            DmaOperationKind::Copied => {
                // Copy path for Flash data (e.g., Init commands) into SRAM bounce buffer.
                if data.len() > bounce_buf_full.len() {
                    panic!("Data length {} exceeds bounce buffer size {}", data.len(), bounce_buf_full.len());
                }
                let bounce_slice = &mut bounce_buf_full[..data.len()];
                bounce_slice.copy_from_slice(data);
                
                let data_ptr = bounce_slice.as_mut_ptr();
                let data_static: &'static mut [u8] = unsafe { core::slice::from_raw_parts_mut(data_ptr, data.len()) };
                DmaTxBuf::new(tx_descriptors, data_static).unwrap()
            }
        };

        let mut transfer = self
            .spi
            .take()
            .unwrap()
            .half_duplex_write(data_mode, cmd, addr, transaction.dummy_cycles, data.len(), tx_buf)
            .map_err(|_| esp_hal::spi::Error::Unsupported)?;

        transfer.wait_for_done().await;
        
        let (spi, buf) = transfer.wait();
        let (desc, _) = buf.split();

        self.spi = Some(spi);
        self.tx_descriptors = Some(desc);
        self.bounce_buf = Some(bounce_buf_full);
        Ok(())
    }

    async fn read(
        &mut self,
        transaction: &QspiTransaction,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        let cmd = to_esp_command(&transaction.instruction);
        let addr = to_esp_address(&transaction.address);
        let data_mode = to_esp_data_mode(transaction.data_mode);

        let mut rx_buf = self.rx_buf.take().unwrap();
        rx_buf.set_length(buffer.len());

        let mut transfer = self
            .spi
            .take()
            .unwrap()
            .half_duplex_read(data_mode, cmd, addr, transaction.dummy_cycles, buffer.len(), rx_buf)
            .map_err(|_| esp_hal::spi::Error::Unsupported)?;

        transfer.wait_for_done().await;
        
        let (spi, buf) = transfer.wait();
        
        buffer.copy_from_slice(&buf.as_slice()[..buffer.len()]);

        self.spi = Some(spi);
        self.rx_buf = Some(buf);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Type-conversion helpers (display-driver types → esp-hal types)
// ---------------------------------------------------------------------------

/// Check if a slice is located in internal SRAM (0x3FC0_0000 - 0x3FFF_FFFF).
fn is_slice_in_dram(data: &[u8]) -> bool {
    let ptr = data.as_ptr() as usize;
    ptr >= 0x3FC0_0000 && ptr < 0x4000_0000
}

enum DmaOperationKind {
    /// The entire slice must be copied into the internal bounce buffer first
    Copied,
    /// The slice can be transferred directly without copying
    InPlace,
}

impl DmaOperationKind {
    fn for_write(buffer: &[u8]) -> Self {
        if is_slice_in_dram(buffer) {
            DmaOperationKind::InPlace
        } else {
            DmaOperationKind::Copied
        }
    }
}

/// Map [`LineMode`] to esp-hal [`DataMode`].
fn to_esp_data_mode(mode: LineMode) -> DataMode {
    match mode {
        LineMode::None | LineMode::Single => DataMode::Single,
        LineMode::Dual => DataMode::Dual,
        LineMode::Quad => DataMode::Quad,
    }
}

/// Map an optional instruction [`PhaseConfig`] to esp-hal [`Command`].
fn to_esp_command(phase: &Option<PhaseConfig>) -> Command {
    match phase {
        None => Command::None,
        Some(cfg) => {
            let mode = to_esp_data_mode(cfg.mode);
            match cfg.bytes_len {
                1 => Command::_8Bit(cfg.value as u16, mode),
                2 => Command::_16Bit(cfg.value as u16, mode),
                _ => Command::_8Bit(cfg.value as u16, mode),
            }
        }
    }
}

/// Map an optional address [`PhaseConfig`] to esp-hal [`Address`].
fn to_esp_address(phase: &Option<PhaseConfig>) -> Address {
    match phase {
        None => Address::None,
        Some(cfg) => {
            let mode = to_esp_data_mode(cfg.mode);
            match cfg.bytes_len {
                1 => Address::_8Bit(cfg.value, mode),
                2 => Address::_16Bit(cfg.value, mode),
                3 => Address::_24Bit(cfg.value, mode),
                4 => Address::_32Bit(cfg.value, mode),
                _ => Address::_24Bit(cfg.value, mode),
            }
        }
    }
}

// (Removed custom AsyncDelay in favor of embassy_time::Delay)

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
    static mut TX_DESCRIPTORS: [esp_hal::dma::DmaDescriptor; 8] = [esp_hal::dma::DmaDescriptor::EMPTY; 8];
    // Bounce buffer for Flash-based initialization commands
    static mut BOUNCE_BUF: [u8; 256] = [0; 256];

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
        tx_descriptors: Some(unsafe { &mut *core::ptr::addr_of_mut!(TX_DESCRIPTORS) }),
        bounce_buf: Some(unsafe { &mut *core::ptr::addr_of_mut!(BOUNCE_BUF) }),
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
