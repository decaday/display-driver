#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START ST7789 ROTATION DEMO");

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

    let spi = Spi::new_txonly(p.SPI4, p.PE12, p.PE14, p.DMA1_CH0, spi_config);

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
                    stm32h7b0_examples::draw_rotation_scene(&mut fb, P_WIDTH, P_HEIGHT, rot_str);

                    // STM32's DMA can send at most 0xFFFF length data at a time.
                    // So we need to split the transfer into two chunks.
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
                    stm32h7b0_examples::draw_rotation_scene(&mut fb, P_HEIGHT, P_WIDTH, rot_str);

                    // STM32's DMA can send at most 0xFFFF length data at a time.
                    // So we need to split the transfer into two chunks.
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
