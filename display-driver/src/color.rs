#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Color format used by the display.
pub enum ColorFormat {
    /// 1-bit per pixel (Monochrome).
    Binary,
    /// 2-bit grayscale.
    Gray2,
    /// 4-bit grayscale.
    Gray4,
    /// 8-bit grayscale.
    Gray8,
    /// 16-bit RGB565.
    RGB565,
    /// 18-bit RGB666.
    RGB666,
    /// 24-bit RGB888.
    RGB888,
}

impl ColorFormat {
    /// Returns the number of bits per pixel for this format.
    pub fn size_bits(self) -> u8 {
        match self {
            ColorFormat::Binary => 1,
            ColorFormat::Gray2 => 2,
            ColorFormat::Gray4 => 4,
            ColorFormat::Gray8 => 8,
            ColorFormat::RGB565 => 16,
            ColorFormat::RGB666 => 18,
            ColorFormat::RGB888 => 24,
        }
    }

    pub fn size_bytes(self) -> u8 {
        match self {
            ColorFormat::Binary => 1,
            ColorFormat::Gray2 => 1,
            ColorFormat::Gray4 => 1,
            ColorFormat::Gray8 => 1,
            ColorFormat::RGB565 => 2,
            ColorFormat::RGB666 => 3,
            ColorFormat::RGB888 => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorType {
    Gray(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, PartialEq, Eq)]
pub struct SolidColor {
    pub raw: [u8; 3],
    pub format: ColorFormat,
    pub color: ColorType,
}

#[cfg(feature = "embedded-graphics")]
mod eg_impls {
    use super::*;
    use embedded_graphics_core::pixelcolor::{
        raw::ToBytes, PixelColor, Rgb565, Rgb666, Rgb888, RgbColor,
    };

    impl<'a> From<Rgb565> for SolidColor {
        fn from(value: Rgb565) -> Self {
            let mut raw = [0u8; 3];
            raw[0..2].copy_from_slice(&<Rgb565 as PixelColor>::Raw::from(value).to_be_bytes());
            SolidColor {
                raw,
                format: ColorFormat::RGB565,
                color: ColorType::Rgb(value.r(), value.g(), value.b()),
            }
        }
    }

    #[cfg(feature = "embedded-graphics")]
    impl<'a> From<Rgb666> for SolidColor {
        fn from(value: Rgb666) -> Self {
            let mut raw = [0u8; 3];
            raw.copy_from_slice(&<Rgb666 as PixelColor>::Raw::from(value).to_be_bytes());
            SolidColor {
                raw,
                format: ColorFormat::RGB666,
                color: ColorType::Rgb(value.r(), value.g(), value.b()),
            }
        }
    }

    #[cfg(feature = "embedded-graphics")]
    impl<'a> From<Rgb888> for SolidColor {
        fn from(value: Rgb888) -> Self {
            let mut raw = [0u8; 3];
            raw.copy_from_slice(&<Rgb888 as PixelColor>::Raw::from(value).to_be_bytes());
            SolidColor {
                raw,
                format: ColorFormat::RGB888,
                color: ColorType::Rgb(value.r(), value.g(), value.b()),
            }
        }
    }
}
