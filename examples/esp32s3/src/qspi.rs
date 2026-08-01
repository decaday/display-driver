//! QSPI Device Adapter for esp-hal
//!
//! `esp-hal`'s `DmaTxBuf` requires an owned buffer. However, `display-driver-qspi`
//! and `embedded-hal` API designs both use `&[u8]`. Furthermore, the ESP32 DMA
//! only supports data originating from DRAM.
//!
//! Therefore, we need to check the memory address to determine if copying is necessary,
//! and use pointer casting to avoid additional copying when data is already in DRAM.
//! [esp-hal internally uses a similar approach to implement `embedded-hal`](https://github.com/esp-rs/esp-hal/blob/8cbe014242843d5b4a9ce14d018f20f16e2ebf18/esp-hal/src/spi/master/dma.rs#L434-L442)

use display_driver_qspi::{LineMode, PhaseConfig, QspiDevice, QspiTransaction};
use esp_hal::{
    dma::{DmaRxBuf, DmaTxBuf},
    spi::master::{Address, Command, DataMode},
};

/// Wraps an esp-hal `SpiDma<Async>` to implement the [`QspiDevice`] trait
/// required by `QspiDisplayBus`.
pub struct EspHalQspiDevice<'d> {
    pub spi: Option<esp_hal::spi::master::SpiDma<'d, esp_hal::Async>>,
    pub rx_buf: Option<DmaRxBuf>,
    pub tx_descriptors: Option<&'static mut [esp_hal::dma::DmaDescriptor]>,
    pub bounce_buf: Option<&'static mut [u8]>,
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
            .map_err(|e| {
                esp_println::println!("half_duplex_write error: {:?}", e);
                esp_hal::spi::Error::Unsupported
            })?;

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
