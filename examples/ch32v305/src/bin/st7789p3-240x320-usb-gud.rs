#![no_std]
#![no_main]

use ch32_hal as hal;
use display_driver::{
    Area, BusBytesIo, ColorFormat, DisplayDriver, DisplayError, LCDResetOption, Orientation, Panel,
};
use display_driver_spi::SpiDisplayBus;
use display_driver_st7789::{spec::generic::Generic240x320P3Type1, St7789};
use embassy_executor::Spawner;
use embassy_futures::{
    join::join,
    select::{select, Either},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Delay;
use embassy_usb::{
    class::gud::device::{
        self as gud, BufferInfo, ConnectorState, DisplayMode, GudClass, GudConnector, GudEvent,
        PixelFormat,
    },
    driver::{Driver, EndpointError},
    Builder, Handler, UsbDeviceSpeed,
};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_hal_bus::spi::ExclusiveDevice;
use hal::{
    gpio::{Level, Output, Speed},
    rcc::{
        AHBPrescaler, APBPrescaler, HsPll, HsPllPrescaler, HsPllSource, Pll, PllMul, PllPreDiv,
        PllSource, Sysclk,
    },
    spi::{self, Spi},
    time::Hertz,
    usb::EndpointDataBuffer,
    usbhs,
};
use panic_halt as _;

hal::bind_interrupts!(struct Irqs {
    USBHS => usbhs::InterruptHandler<hal::peripherals::USBHS>;
    USBHS_WKUP => usbhs::WakeupInterruptHandler<hal::peripherals::USBHS>;
});

const WIDTH: u32 = 240;
const HEIGHT: u32 = 320;
const PACKET_SIZE: usize = 512;
// Two staging halves, not a limit on the host's damage rectangle size.
const CHUNK_SIZE: usize = 4 * 1024;
const MODES: [DisplayMode; 1] = [DisplayMode {
    clock: 5000,
    hdisplay: WIDTH as u16,
    hsync_start: 248,
    hsync_end: 256,
    htotal: 272,
    vdisplay: HEIGHT as u16,
    vsync_start: 324,
    vsync_end: 328,
    vtotal: 340,
    flags: 0,
    preferred: true,
}];

// The pinned HAL does not wake an already pending OUT read on bus reset.
// Cancel that read only after the USB stack has invalidated its configuration.
struct UsbReset<'a>(&'a Signal<CriticalSectionRawMutex, ()>);
impl Handler for UsbReset<'_> {
    fn reset(&mut self) {
        self.0.signal(());
    }
    fn configured(&mut self, configured: bool) {
        if !configured {
            self.0.signal(());
        }
    }
}

#[derive(Default)]
struct DisplayState {
    configured: bool,
    committed: bool,
    controller_enabled: bool,
    display_enabled: bool,
    has_frame: bool,
}
impl DisplayState {
    fn apply(&mut self, event: GudEvent) {
        match event {
            GudEvent::Reset | GudEvent::Configured(false) => *self = Self::default(),
            GudEvent::Configured(true) => self.configured = true,
            GudEvent::ControllerEnable(enabled) => self.controller_enabled = enabled,
            GudEvent::DisplayEnable(enabled) => self.display_enabled = enabled,
            _ => {}
        }
    }
    fn visible(&self) -> bool {
        self.configured
            && self.committed
            && self.controller_enabled
            && self.display_enabled
            && self.has_frame
    }
}

// SET_BUFFER validates compression/size in the class, but not rectangle geometry.
fn buffer_area(info: &BufferInfo) -> Option<Area> {
    if info.compression != 0
        || info.width == 0
        || info.height == 0
        || info.x.checked_add(info.width)? > WIDTH
        || info.y.checked_add(info.height)? > HEIGHT
        || info.length != info.width.checked_mul(info.height)?.checked_mul(2)?
    {
        return None;
    }
    Some(Area::new(
        info.x as u16,
        info.y as u16,
        info.width as u16,
        info.height as u16,
    ))
}

fn rgb565_to_spi(bytes: &mut [u8]) {
    for pixel in bytes.chunks_exact_mut(2) {
        pixel.swap(0, 1);
    }
}

#[derive(Debug)]
enum ReceiveError<E> {
    Reset,
    Usb(EndpointError),
    Display(DisplayError<E>),
    InvalidPacket,
}

async fn receive_chunk<'d, D: Driver<'d>, E>(
    class: &mut GudClass<'d, D>,
    remaining: &mut usize,
    bytes: &mut [u8; CHUNK_SIZE],
    mut buffered: usize,
    reset: &Signal<CriticalSectionRawMutex, ()>,
) -> Result<usize, ReceiveError<E>> {
    if reset.signaled() {
        return Err(ReceiveError::Reset);
    }
    // The HAL requires full-packet receive capacity even for a short final
    // packet. Stop filling while another full packet would not fit.
    while *remaining != 0 && bytes.len() - buffered >= PACKET_SIZE {
        let n = match select(
            reset.wait(),
            class.read_packet(&mut bytes[buffered..buffered + PACKET_SIZE]),
        )
        .await
        {
            Either::First(()) => return Err(ReceiveError::Reset),
            Either::Second(result) => result.map_err(ReceiveError::Usb)?,
        };
        if n == 0 || n > *remaining {
            return Err(ReceiveError::InvalidPacket);
        }
        *remaining -= n;
        buffered += n;
    }
    Ok(buffered)
}

async fn stream_buffer<'d, D, B, P>(
    class: &mut GudClass<'d, D>,
    info: &BufferInfo,
    area: Option<Area>,
    display: &mut DisplayDriver<B, P>,
    buffers: &mut [[u8; CHUNK_SIZE]; 2],
    reset: &Signal<CriticalSectionRawMutex, ()>,
) -> Result<bool, ReceiveError<B::Error>>
where
    D: Driver<'d>,
    B: BusBytesIo,
    P: Panel<B>,
{
    let mut remaining = info.transfer_size() as usize;
    let [mut current, mut next] = buffers.each_mut();

    // Drain invalid/uncommitted updates without issuing any LCD commands.
    let Some(area) = area else {
        while remaining != 0 {
            receive_chunk::<D, B::Error>(class, &mut remaining, current, 0, reset).await?;
        }
        return Ok(false);
    };

    let mut buffered =
        receive_chunk::<D, B::Error>(class, &mut remaining, current, 0, reset).await?;
    if buffered == 0 {
        return Ok(false);
    }
    display
        .set_window(area)
        .await
        .map_err(ReceiveError::Display)?;
    display
        .bus
        .write_cmd_bytes(&P::PIXEL_WRITE_CMD[..P::CMD_LEN])
        .await
        .map_err(|e| ReceiveError::Display(DisplayError::BusError(e)))?;

    loop {
        let complete = buffered & !1;
        let carried = buffered & 1;
        if remaining == 0 && carried != 0 {
            return Err(ReceiveError::InvalidPacket);
        }
        if carried != 0 {
            // Preserve the low byte of a split USB RGB565 pixel before swapping.
            next[0] = current[complete];
        }
        rgb565_to_spi(&mut current[..complete]);

        // Disjoint buffers let SPI DMA drain one half while USB fills the other.
        // join (not select/try_join) finishes in-flight SPI even on USB reset/error.
        let (written, received) = join(
            display.bus.write_data_bytes(&current[..complete]),
            receive_chunk::<D, B::Error>(class, &mut remaining, next, carried, reset),
        )
        .await;
        written.map_err(|e| ReceiveError::Display(DisplayError::BusError(e)))?;
        buffered = received?;
        // Reset may have arrived after reception completed but before SPI did.
        if reset.signaled() {
            return Err(ReceiveError::Reset);
        }
        if buffered == 0 {
            return Ok(true);
        }
        core::mem::swap(&mut current, &mut next);
    }
}

#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = hal::Config::default();
    // HSI 8 MHz * 18 = 144 MHz PCLK1; SPI /2 = 72 MHz.
    // Deliberate panel overclock: ST7789P3 specifies at most 62.5 MHz.
    config.rcc = hal::rcc::Config {
        sys: Sysclk::PLL,
        pll_src: PllSource::HSI,
        pll: Some(Pll {
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL18,
        }),
        pllx: None,
        // HSI/2 supplies the supported 4 MHz reference for the USBHS PLL.
        hspll_src: HsPllSource::HSI,
        hspll: Some(HsPll {
            pre: HsPllPrescaler::DIV2,
        }),
        ahb_pre: AHBPrescaler::DIV1,
        apb1_pre: APBPrescaler::DIV1,
        apb2_pre: APBPrescaler::DIV1,
        ..Default::default()
    };
    let p = hal::init(config);
    let mut backlight = Output::new(p.PB4, Level::High, Speed::Low);
    let cs = Output::new(p.PC7, Level::High, Speed::High);
    let rs = Output::new(p.PA15, Level::High, Speed::High);
    let rst = Output::new(p.PC14, Level::High, Speed::Low);
    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz::mhz(72);
    spi_config.mode = embedded_hal::spi::MODE_0;
    let spi = Spi::new_txonly::<0>(p.SPI3, p.PB3, p.PB5, p.DMA2_CH2, spi_config);
    let bus = SpiDisplayBus::new(ExclusiveDevice::new_no_delay(spi, cs).unwrap(), rs);
    let panel = St7789::<Generic240x320P3Type1, _, _>::new(LCDResetOption::new_pin(rst));
    let mut display = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .with_orientation(Orientation::Deg180)
        .init(&mut Delay)
        .await
        .unwrap();
    display
        .fill_screen_batch::<480>(Rgb565::BLACK.into())
        .await
        .unwrap();

    // PB7 D+ / PB6 D- are USBHS; PA12 remains available to the board's touch IC.
    let mut ep_buffers: [EndpointDataBuffer<512>; 2] =
        core::array::from_fn(|_| EndpointDataBuffer::default());
    let usb_driver = usbhs::Driver::new(p.USBHS, Irqs, p.PB7, p.PB6, &mut ep_buffers);
    let mut usb_config = embassy_usb::Config::new(gud::GUD_VENDOR_ID, gud::GUD_PRODUCT_ID);
    usb_config.manufacturer = Some("display-driver");
    usb_config.product = Some("CH32V305 ST7789P3 GUD");
    usb_config.max_speed = UsbDeviceSpeed::High;
    usb_config.device_class = 0;
    usb_config.device_sub_class = 0;
    usb_config.device_protocol = 0;
    usb_config.composite_with_iads = false;
    usb_config.max_power = 250;

    let mut mode_storage = MODES;
    let connector_state = ConnectorState::new(&mut mode_storage, &mut []);
    connector_state
        .update(gud::CONNECTOR_STATUS_CONNECTED, &MODES, None)
        .unwrap();
    let connectors = [GudConnector {
        connector_type: gud::CONNECTOR_TYPE_PANEL,
        flags: 0,
        state: &connector_state,
        properties: &[],
        tv_mode_values: None,
    }];
    let gud_config = gud::Config {
        max_packet_size: PACKET_SIZE as u16,
        min_width: WIDTH,
        max_width: WIDTH,
        min_height: HEIGHT,
        max_height: HEIGHT,
        // No host rectangle cap; two 4 KiB halves bound staging RAM.
        max_buffer_size: 0,
        formats: &[PixelFormat::Rgb565],
        supported_rotations: 0,
        connectors: &connectors,
        // GUD delegates mode acceptance to the device. Keep this fixed panel's
        // advertised timings/flags; preferred is host metadata, not a timing.
        validate_mode_blocking: |connector, mode| {
            connector == 0
                && *mode
                    == DisplayMode {
                        preferred: mode.preferred,
                        ..MODES[0]
                    }
        },
    };

    let mut config_descriptor = [0; 128];
    let mut bos_descriptor = [0; 64];
    let mut msos_descriptor = [0; 0];
    let mut control_buffer = [0; 256];
    let reset = Signal::new();
    let mut reset_handler = UsbReset(&reset);
    let mut gud_state = gud::State::new();
    let mut builder = Builder::new(
        usb_driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buffer,
    );
    builder.handler(&mut reset_handler);
    let mut class = GudClass::new(&mut builder, &mut gud_state, &gud_config);
    let mut usb = builder.build();

    let display_task = async {
        let mut state = DisplayState::default();
        let mut buffers = [[0; CHUNK_SIZE]; 2];
        loop {
            let event = class.next_event().await;
            state.apply(event);
            match event {
                GudEvent::Reset | GudEvent::Configured(false) => reset.reset(),
                GudEvent::StateCommit => {
                    // A checked state is only a proposal. Use the class's committed
                    // snapshot; a concurrent USB reset may already have cleared it.
                    state.committed = class.committed_state().is_some();
                    if !state.committed {
                        state.has_frame = false;
                    }
                }
                GudEvent::ConnectorForceDetect(connector) => {
                    connectors[connector as usize]
                        .state
                        .update(gud::CONNECTOR_STATUS_CONNECTED, &MODES, None)
                        .unwrap();
                }
                GudEvent::Buffer(info) => {
                    let area = if state.committed {
                        buffer_area(&info)
                    } else {
                        None
                    };
                    match stream_buffer(&mut class, &info, area, &mut display, &mut buffers, &reset)
                        .await
                    {
                        Ok(wrote) => state.has_frame |= wrote,
                        Err(ReceiveError::Reset | ReceiveError::Usb(EndpointError::Disabled)) => {
                            state = DisplayState::default();
                        }
                        Err(ReceiveError::Display(error)) => {
                            backlight.set_high();
                            panic!("GUD display write failed: {:?}", error);
                        }
                        Err(_) => {
                            backlight.set_high();
                            panic!("GUD bulk transfer failed");
                        }
                    }
                }
                _ => {}
            }
            if state.visible() {
                backlight.set_low();
            } else {
                backlight.set_high();
            }
        }
    };
    join(usb.run(), display_task).await;
}
