#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use micromath::F32Ext;

use display_driver::{panel::reset::LCDResetOption, ColorFormat};
use display_driver::{Area, DisplayDriver, FrameControl, Orientation};
use display_driver_gc9a01::{spec::Generic240x240Type1, Gc9a01};
use display_driver_spi::SpiDisplayBus;

use tjpgd_rs::JpegDecoder;
use tjpgd_rs::types::{PixelFormat, Scale};

use embedded_graphics::{
    geometry::{Point, Size, OriginDimensions},
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle},
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::*,
    text::{Alignment, Text},
};

const WIDTH: usize = 240;
const HEIGHT: usize = 240;

embassy_stm32::bind_interrupts!(struct Irqs {
    DMA1_STREAM0 => embassy_stm32::dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH0>;
});

#[repr(C, align(32))]
struct AlignedBuffer(pub [u8; WIDTH * HEIGHT * 2]);
static mut FB: AlignedBuffer = AlignedBuffer([0; WIDTH * HEIGHT * 2]);

#[repr(C, align(32))]
struct CatBuffer(pub [u8; 120 * 120 * 2]);
static mut CAT_SRC: CatBuffer = CatBuffer([0; 120 * 120 * 2]);

struct RawFb<'a> {
    data: &'a mut [u8],
}
impl<'a> OriginDimensions for RawFb<'a> {
    fn size(&self) -> Size { Size::new(WIDTH as u32, HEIGHT as u32) }
}
impl<'a> DrawTarget for RawFb<'a> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>
    {
        for Pixel(Point { x, y }, color) in pixels.into_iter() {
            if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                let idx = (y as usize * WIDTH + x as usize) * 2;
                let raw: u16 = RawU16::from(color).into_inner();
                let bytes = raw.to_be_bytes();
                self.data[idx] = bytes[0];
                self.data[idx+1] = bytes[1];
            }
        }
        Ok(())
    }
}

struct Danmaku {
    text: &'static str,
    x: f32,
    y: i32,
    vx: f32,
    color: Rgb565,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("START GC9A01 CRAZY CAT DEMO");

    let config = stm32h7b0_examples::configure_rcc();
    let p = embassy_stm32::init(config);

    let dc = Output::new(p.PE13, Level::Low, Speed::High);
    let cs = Output::new(p.PE9, Level::High, Speed::High);
    let rst = Output::new(p.PE15, Level::High, Speed::High);
    let _lcd_led = Output::new(p.PE10, Level::Low, Speed::Low);

    let mut spi_config: spi::Config = Default::default();
    spi_config.frequency = Hertz(40_000_000); 

    let spi = Spi::new_txonly(p.SPI4, p.PE12, p.PE14, p.DMA1_CH0, Irqs, spi_config);
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let bus = SpiDisplayBus::new(spi_device, dc);
    let panel = Gc9a01::<Generic240x240Type1, _, _>::new(LCDResetOption::new_pin(rst));

    let mut disp = DisplayDriver::builder(bus, panel)
        .with_color_format(ColorFormat::RGB565)
        .with_orientation(Orientation::Deg180)
        .init(&mut embassy_time::Delay)
        .await
        .unwrap();
    info!("Display initialized.");

    let image_data = include_bytes!("../../../assets/cat_flower.jpg");
    let mut reader = &image_data[..];
    let mut pool = [0u8; 3500];

    let mut decoder = JpegDecoder::new(&mut pool[..], &mut reader).unwrap();
    
    let cat_buf = unsafe { &mut *core::ptr::addr_of_mut!(CAT_SRC.0) };
    let fb_buf = unsafe { &mut *core::ptr::addr_of_mut!(FB.0) };

    unsafe {
        let mut cp = cortex_m::Peripherals::steal();
        cp.SCB.enable_icache();
        cp.SCB.enable_dcache(&mut cp.CPUID);
    }

    let start_time = embassy_time::Instant::now();
    decoder.decode_to_framebuffer(
        cat_buf,
        120, 
        120, 
        0,
        0,
        PixelFormat::RGB565,
        Scale::None,
    ).unwrap();
    let elapsed = start_time.elapsed().as_millis();
    info!("JPEG Decoded in {} ms!", elapsed);

    // Disable D-Cache before animation loop to ensure DMA coherence!
    unsafe {
        let mut cp = cortex_m::Peripherals::steal();
        cp.SCB.disable_dcache(&mut cp.CPUID);
    }

    let mut frames = 0;
    let mut last_time = embassy_time::Instant::now();

    let mut danmakus = [
        Danmaku { text: "MEOW!!!", x: 240.0, y: 30, vx: -3.0, color: Rgb565::RED },
        Danmaku { text: "INHALE THE CAT!", x: 400.0, y: 70, vx: -4.5, color: Rgb565::BLUE },
        Danmaku { text: "CAT IS JUSTICE", x: -100.0, y: 190, vx: 5.0, color: Rgb565::new(0, 63, 0) }, // Green
        Danmaku { text: "CATS RULE THE WORLD", x: 300.0, y: 150, vx: -2.0, color: Rgb565::MAGENTA },
        Danmaku { text: "SNIFF SNIFF", x: 100.0, y: 220, vx: 3.5, color: Rgb565::YELLOW },
        Danmaku { text: "GIMME TUNA!", x: 500.0, y: 100, vx: -5.5, color: Rgb565::CYAN },
    ];

    loop {
        fb_buf.fill(0xFF);

        // Crazy Math
        let time = frames as f32;
        let scale = 1.0 + 0.6 * (time * 0.05).sin();
        let angle = time * 0.1 + (time * 0.03).sin() * 3.0; // Oscillating rotation
        
        let (sin_a, cos_a) = angle.sin_cos();

        for y in 0usize..240 {
            for x in 0usize..240 {
                let dx = x as f32 - 120.0;
                let dy = y as f32 - 120.0;

                let src_x = (dx * cos_a + dy * sin_a) / scale + 60.0;
                let src_y = (-dx * sin_a + dy * cos_a) / scale + 60.0;

                if src_x >= 0.0 && src_x < 120.0 && src_y >= 0.0 && src_y < 120.0 {
                    let ix = src_x as usize;
                    let iy = src_y as usize;
                    let src_idx = (iy * 120 + ix) * 2;
                    let dst_idx = (y * 240 + x) * 2;

                    fb_buf[dst_idx] = cat_buf[src_idx];
                    fb_buf[dst_idx + 1] = cat_buf[src_idx + 1];
                }
            }
        }

        // Draw Danmaku
        let mut draw_target = RawFb { data: fb_buf };
        for d in danmakus.iter_mut() {
            // Rainbow colors!
            let r = ( (time * 0.1 + d.x * 0.01).sin() * 15.0 + 15.0 ) as u8;
            let g = ( (time * 0.1 + d.x * 0.01 + 2.0).sin() * 31.0 + 31.0 ) as u8;
            let b = ( (time * 0.1 + d.x * 0.01 + 4.0).sin() * 15.0 + 15.0 ) as u8;
            d.color = Rgb565::new(r, g, b);

            let style = MonoTextStyle::new(&FONT_9X18_BOLD, d.color);
            Text::with_alignment(d.text, Point::new(d.x as i32, d.y), style, Alignment::Left)
                .draw(&mut draw_target)
                .unwrap();

            d.x += d.vx;
            if d.vx < 0.0 && d.x < -150.0 {
                d.x = 250.0 + (frames % 100) as f32;
            } else if d.vx > 0.0 && d.x > 250.0 {
                d.x = -150.0 - (frames % 100) as f32;
            }
        }

        // (Removed clean_dcache_by_slice because D-Cache is now completely disabled)

        let chunk_lines = 120;
        let chunk_bytes = chunk_lines * 240 * 2;

        disp.write_pixels(
            Area::from_origin(240, chunk_lines as u16),
            FrameControl::new_first(),
            &fb_buf[0..chunk_bytes],
        ).await.unwrap();

        disp.write_pixels(
            Area::new(0, chunk_lines as u16, 240, chunk_lines as u16),
            FrameControl::new_last(),
            &fb_buf[chunk_bytes..chunk_bytes * 2],
        ).await.unwrap();

        frames += 1;
        if frames % 60 == 0 {
            let elapsed = last_time.elapsed().as_millis() as f32 / 1000.0;
            let fps = 60.0 / elapsed;
            info!("FPS: {}", fps);
            last_time = embassy_time::Instant::now();
        }
    }
}
