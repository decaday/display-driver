use display_driver_qspi::{LineMode, QspiDevice, QspiTransaction};
use embassy_stm32::ospi::{AddressSize, DummyCycles, Ospi, OspiError, OspiWidth, TransferConfig};

/// Wrapper to implement `QspiDevice` for `embassy-stm32`'s `Ospi` peripheral.
pub struct Stm32OspiDevice<'d, T: embassy_stm32::ospi::Instance> {
    pub ospi: Ospi<'d, T, embassy_stm32::mode::Async>,
}

fn map_line_mode(mode: LineMode) -> OspiWidth {
    match mode {
        LineMode::None => OspiWidth::NONE,
        LineMode::Single => OspiWidth::SING,
        LineMode::Dual => OspiWidth::DUAL,
        LineMode::Quad => OspiWidth::QUAD,
    }
}

impl<'d, T: embassy_stm32::ospi::Instance> Stm32OspiDevice<'d, T> {
    fn map_transaction(transaction: &QspiTransaction) -> TransferConfig {
        let mut cfg = TransferConfig::default();

        if let Some(inst) = &transaction.instruction {
            cfg.iwidth = map_line_mode(inst.mode);
            cfg.instruction = Some(inst.value);
            cfg.isize = match inst.bytes_len {
                1 => AddressSize::_8Bit,
                2 => AddressSize::_16Bit,
                3 => AddressSize::_24bit,
                4 => AddressSize::_32bit,
                _ => AddressSize::_8Bit,
            };
        }

        if let Some(addr) = &transaction.address {
            cfg.adwidth = map_line_mode(addr.mode);
            cfg.address = Some(addr.value);
            cfg.adsize = match addr.bytes_len {
                1 => AddressSize::_8Bit,
                2 => AddressSize::_16Bit,
                3 => AddressSize::_24bit,
                4 => AddressSize::_32bit,
                _ => AddressSize::_24bit,
            };
        }

        cfg.dwidth = map_line_mode(transaction.data_mode);
        
        cfg.dummy = match transaction.dummy_cycles {
            0 => DummyCycles::_0,
            1 => DummyCycles::_1,
            2 => DummyCycles::_2,
            3 => DummyCycles::_3,
            4 => DummyCycles::_4,
            5 => DummyCycles::_5,
            6 => DummyCycles::_6,
            7 => DummyCycles::_7,
            8 => DummyCycles::_8,
            9 => DummyCycles::_9,
            10 => DummyCycles::_10,
            11 => DummyCycles::_11,
            12 => DummyCycles::_12,
            13 => DummyCycles::_13,
            14 => DummyCycles::_14,
            15 => DummyCycles::_15,
            16 => DummyCycles::_16,
            _ => DummyCycles::_0,
        };

        cfg
    }
}

impl<'d, T: embassy_stm32::ospi::Instance> QspiDevice for Stm32OspiDevice<'d, T> {
    type Error = OspiError;

    async fn write(
        &mut self,
        transaction: &QspiTransaction,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let mut cfg = Self::map_transaction(transaction);
        if data.is_empty() {
            cfg.dwidth = OspiWidth::NONE;
            self.ospi.blocking_command(&cfg)
        } else {
            self.ospi.write(data, cfg).await
        }
    }

    async fn read(
        &mut self,
        transaction: &QspiTransaction,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut cfg = Self::map_transaction(transaction);
        if buffer.is_empty() {
            cfg.dwidth = OspiWidth::NONE;
            self.ospi.blocking_command(&cfg)
        } else {
            self.ospi.read(buffer, cfg).await
        }
    }
}
