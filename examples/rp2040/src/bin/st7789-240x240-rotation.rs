#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::{self, Spi};
use embassy_time::Timer;

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    pixelcolor::{
        raw::{BigEndian, RawU16},
        Rgb565,
    },
};

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{Area, DisplayDriver, FrameControl, Orientation};
use display_driver_spi::SpiDisplayBus;
use display_driver_st7789::{spec::generic::Generic240x240Type1, spec::PanelSpec, St7789};

// Native dimensions (Portrait 0 degree)
const P_WIDTH: usize = Generic240x240Type1::PHYSICAL_WIDTH as usize;
const P_HEIGHT: usize = Generic240x240Type1::PHYSICAL_HEIGHT as usize;

embassy_rp::bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<embassy_rp::peripherals::DMA_CH0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START ST7789 ROTATION DEMO");

    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Wiring (RP2040 -> ST7789 240x240 display)
    // 3V3  -> VCC     | GND -> GND
    // PIN_18 -> SCL/CLK | PIN_19 -> SDA/DIN
    // PIN_17 -> CS      | PIN_16 -> DC
    // PIN_20 -> RST     | PIN_21 -> BL/LED
    let dc = Output::new(p.PIN_16, Level::Low);
    let cs = Output::new(p.PIN_17, Level::High);
    let rst = Output::new(p.PIN_20, Level::High);
    let _lcd_led = Output::new(p.PIN_21, Level::High);

    let mut spi_config: spi::Config = Default::default();
    spi_config.frequency = 10_000_000;
    spi_config.phase = spi::Phase::CaptureOnFirstTransition;
    spi_config.polarity = spi::Polarity::IdleLow;

    let spi = Spi::new_txonly(p.SPI0, p.PIN_18, p.PIN_19, p.DMA_CH0, Irqs, spi_config);

    // Create the SPI Bus
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let bus = SpiDisplayBus::new(spi_device, dc);

    // Create the Panel
    let panel = St7789::<Generic240x240Type1, _, _>::new(LCDResetOption::new_pin(rst));

    // Create and initialize the Driver using builder
    info!("Initializing display...");
    let mut disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();
    info!("Display initialized.");

    // Loop orientations
    loop {
        for rot in [
            Orientation::Deg0,
            Orientation::Deg90,
            Orientation::Deg180,
            Orientation::Deg270,
        ] {
            let rot_str = match rot {
                Orientation::Deg0 => "Deg 0",
                Orientation::Deg90 => "Deg 90",
                Orientation::Deg180 => "Deg 180",
                Orientation::Deg270 => "Deg 270",
            };
            info!("Rotating to {}", rot_str);
            disp.set_orientation(rot).await.unwrap();

            // Create framebuffer on stack and draw
            match rot {
                Orientation::Deg0 | Orientation::Deg180 => {
                    let mut fb = Framebuffer::<
                        Rgb565,
                        RawU16,
                        BigEndian,
                        P_WIDTH,
                        P_HEIGHT,
                        { buffer_size::<Rgb565>(P_WIDTH, P_HEIGHT) },
                    >::new();
                    display_driver::eg::utils::draw_rotation_scene(&mut fb, P_WIDTH, P_HEIGHT, rot_str);

                    // DMA can send at most some limit, splitting just in case
                    let data = fb.data();
                    let (first, second) = data.split_at(data.len() / 2);

                    // Send first half
                    disp.write_pixels(
                        Area::from_origin(P_WIDTH as u16, (P_HEIGHT / 2) as u16),
                        FrameControl::new_first(),
                        first,
                    )
                    .await
                    .unwrap();

                    // Send second half
                    disp.write_pixels(
                        Area::new(
                            0,
                            (P_HEIGHT / 2) as u16,
                            P_WIDTH as u16,
                            (P_HEIGHT / 2) as u16,
                        ),
                        FrameControl::new_last(),
                        second,
                    )
                    .await
                    .unwrap();
                }
                Orientation::Deg90 | Orientation::Deg270 => {
                    let mut fb = Framebuffer::<
                        Rgb565,
                        RawU16,
                        BigEndian,
                        P_HEIGHT,
                        P_WIDTH,
                        { buffer_size::<Rgb565>(P_HEIGHT, P_WIDTH) },
                    >::new();
                    display_driver::eg::utils::draw_rotation_scene(&mut fb, P_HEIGHT, P_WIDTH, rot_str);

                    // DMA can send at most some limit, splitting just in case
                    let data = fb.data();
                    let (first, second) = data.split_at(data.len() / 2);

                    // Send first half
                    disp.write_pixels(
                        Area::from_origin(P_HEIGHT as u16, (P_WIDTH / 2) as u16),
                        FrameControl::new_first(),
                        first,
                    )
                    .await
                    .unwrap();

                    // Send second half
                    disp.write_pixels(
                        Area::new(
                            0,
                            (P_WIDTH / 2) as u16,
                            P_HEIGHT as u16,
                            (P_WIDTH / 2) as u16,
                        ),
                        FrameControl::new_last(),
                        second,
                    )
                    .await
                    .unwrap();
                }
            }

            Timer::after_secs(3).await;
        }
    }
}
