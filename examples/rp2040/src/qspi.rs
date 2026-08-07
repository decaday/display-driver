//! RP2040 PIO QSPI Device Implementation
//!
//! Implements [`QspiDevice`] from `display-driver-qspi` using RP2040 PIO state machine and DMA.

use display_driver_qspi::{LineMode, QspiDevice, QspiTransaction};
use embassy_futures::join::join;
use embassy_rp::{
    clocks::clk_sys_freq,
    dma,
    gpio::Level,
    pio::{
        program::pio_asm, Common, Config, Direction, Instance, LoadedProgram, PioPin,
        ShiftDirection, StateMachine,
    },
    Peri,
};
use embedded_hal::digital::OutputPin;
use fixed::traits::ToFixed;
use fixed::types::extra::U8;
use fixed::{FixedU32, FixedU64};

/// Error type for RP2040 PIO QSPI operations.
#[derive(Debug, Clone, Copy)]
pub enum PioQspiError {
    /// Operation failed
    Failed,
}

/// RP2040 PIO QSPI Device implementation.
pub struct PioQspiDevice<'d, PIO: Instance, const SM: usize, CS: OutputPin> {
    sm: StateMachine<'d, PIO, SM>,
    cs: CS,
    single_tx_cfg: Config<'d, PIO>,
    quad_tx_cfg: Config<'d, PIO>,
    single_rx_cfg: Config<'d, PIO>,
    single_tx_origin: u8,
    quad_tx_origin: u8,
    single_rx_origin: u8,
    tx_dma: dma::Channel<'d>,
    rx_dma: dma::Channel<'d>,
}

impl<'d, PIO: Instance, const SM: usize, CS: OutputPin> PioQspiDevice<'d, PIO, SM, CS> {
    /// Create a new PIO QSPI device instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new<TxDma: dma::ChannelInstance, RxDma: dma::ChannelInstance>(
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        sclk: Peri<'d, impl PioPin>,
        sio0: Peri<'d, impl PioPin>,
        sio1: Peri<'d, impl PioPin>,
        sio2: Peri<'d, impl PioPin>,
        sio3: Peri<'d, impl PioPin>,
        mut cs: CS,
        tx_dma: Peri<'d, TxDma>,
        rx_dma: Peri<'d, RxDma>,
        irq: impl embassy_rp::interrupt::typelevel::Binding<
            TxDma::Interrupt,
            dma::InterruptHandler<TxDma>,
        > + embassy_rp::interrupt::typelevel::Binding<
            RxDma::Interrupt,
            dma::InterruptHandler<RxDma>,
        > + 'd,
        frequency_hz: u32,
    ) -> Self {
        // Build 3 distinct PIO programs with explicit wrap bounds
        let single_tx_prg = pio_asm!(
            ".side_set 1",
            "out pins, 1 side 0 [1]",
            "nop         side 1 [1]",
        );

        let quad_tx_prg = pio_asm!(
            ".side_set 1",
            "out pins, 4 side 0 [1]",
            "nop         side 1 [1]",
        );

        let single_rx_prg = pio_asm!(
            ".side_set 1",
            "out null, 1 side 0 [1]",
            "in pins, 1  side 1 [1]",
        );

        let single_tx_loaded: LoadedProgram<'d, PIO> = common.load_program(&single_tx_prg.program);
        let quad_tx_loaded: LoadedProgram<'d, PIO> = common.load_program(&quad_tx_prg.program);
        let single_rx_loaded: LoadedProgram<'d, PIO> = common.load_program(&single_rx_prg.program);

        let sclk_pin = common.make_pio_pin(sclk);
        let sio0_pin = common.make_pio_pin(sio0);
        let sio1_pin = common.make_pio_pin(sio1);
        let sio2_pin = common.make_pio_pin(sio2);
        let sio3_pin = common.make_pio_pin(sio3);

        // Ensure consecutive pins for data bus sio0..sio3
        assert_eq!(sio0_pin.pin() + 1, sio1_pin.pin(), "sio0 and sio1 must be consecutive pins");
        assert_eq!(sio1_pin.pin() + 1, sio2_pin.pin(), "sio1 and sio2 must be consecutive pins");
        assert_eq!(sio2_pin.pin() + 1, sio3_pin.pin(), "sio2 and sio3 must be consecutive pins");

        // Calculate PIO clock divider: sys_freq / (frequency_hz * 4)
        let sys_freq = clk_sys_freq().to_fixed::<FixedU64<U8>>();
        let target_freq = (frequency_hz * 4).to_fixed::<FixedU64<U8>>();
        let clkdiv: FixedU32<U8> = (sys_freq / target_freq).to_fixed();

        // 1. Single TX config
        let mut single_tx_cfg = Config::default();
        single_tx_cfg.use_program(&single_tx_loaded, &[&sclk_pin]);
        single_tx_cfg.set_out_pins(&[&sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        single_tx_cfg.set_in_pins(&[&sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        single_tx_cfg.set_set_pins(&[&sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        single_tx_cfg.shift_out.auto_fill = true;
        single_tx_cfg.shift_out.direction = ShiftDirection::Left;
        single_tx_cfg.shift_out.threshold = 8;
        single_tx_cfg.clock_divider = clkdiv;

        // 2. Quad TX config
        let mut quad_tx_cfg = Config::default();
        quad_tx_cfg.use_program(&quad_tx_loaded, &[&sclk_pin]);
        quad_tx_cfg.set_out_pins(&[&sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        quad_tx_cfg.set_in_pins(&[&sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        quad_tx_cfg.set_set_pins(&[&sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        quad_tx_cfg.shift_out.auto_fill = true;
        quad_tx_cfg.shift_out.direction = ShiftDirection::Left;
        quad_tx_cfg.shift_out.threshold = 8;
        quad_tx_cfg.clock_divider = clkdiv;

        // 3. Single RX config
        let mut single_rx_cfg = Config::default();
        single_rx_cfg.use_program(&single_rx_loaded, &[&sclk_pin]);
        single_rx_cfg.set_out_pins(&[&sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        single_rx_cfg.set_in_pins(&[&sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        single_rx_cfg.set_set_pins(&[&sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        single_rx_cfg.shift_in.auto_fill = true;
        single_rx_cfg.shift_in.direction = ShiftDirection::Left;
        single_rx_cfg.shift_in.threshold = 8;
        single_rx_cfg.shift_out.auto_fill = true;
        single_rx_cfg.shift_out.direction = ShiftDirection::Left;
        single_rx_cfg.shift_out.threshold = 8;
        single_rx_cfg.clock_divider = clkdiv;

        sm.set_config(&single_tx_cfg);

        sm.set_pins(Level::Low, &[&sclk_pin, &sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);
        sm.set_pin_dirs(Direction::Out, &[&sclk_pin, &sio0_pin, &sio1_pin, &sio2_pin, &sio3_pin]);

        sm.set_enable(true);
        let _ = cs.set_high();

        let tx_dma_ch = dma::Channel::new(tx_dma, irq);
        let rx_dma_ch = dma::Channel::new(rx_dma, irq);

        Self {
            sm,
            cs,
            single_tx_cfg,
            quad_tx_cfg,
            single_rx_cfg,
            single_tx_origin: single_tx_loaded.origin,
            quad_tx_origin: quad_tx_loaded.origin,
            single_rx_origin: single_rx_loaded.origin,
            tx_dma: tx_dma_ch,
            rx_dma: rx_dma_ch,
        }
    }

    #[inline(always)]
    fn set_mode_single_tx(&mut self) {
        self.sm.set_enable(false);
        self.sm.set_config(&self.single_tx_cfg);
        unsafe { self.sm.exec_jmp(self.single_tx_origin); }
        self.sm.clear_fifos();
        self.sm.set_enable(true);
    }

    #[inline(always)]
    fn set_mode_quad_tx(&mut self) {
        self.sm.set_enable(false);
        self.sm.set_config(&self.quad_tx_cfg);
        unsafe { self.sm.exec_jmp(self.quad_tx_origin); }
        self.sm.clear_fifos();
        self.sm.set_enable(true);
    }

    #[inline(always)]
    fn set_mode_single_rx(&mut self) {
        self.sm.set_enable(false);
        self.sm.set_config(&self.single_rx_cfg);
        unsafe { self.sm.exec_jmp(self.single_rx_origin); }
        self.sm.clear_fifos();
        self.sm.set_enable(true);
    }

    #[inline(always)]
    fn write_byte(&mut self, b: u8) {
        let val_be = u32::from_be_bytes([b, 0, 0, 0]);
        while !self.sm.tx().try_push(val_be) {}
    }

    #[inline(always)]
    fn wait_tx_done(&mut self) {
        while !self.sm.tx().empty() {}
        while !self.sm.tx().stalled() {}
    }
}

impl<'d, PIO: Instance, const SM: usize, CS: OutputPin> QspiDevice for PioQspiDevice<'d, PIO, SM, CS> {
    type Error = PioQspiError;

    async fn write(
        &mut self,
        transaction: &QspiTransaction,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let _ = self.cs.set_low();

        // 1. Instruction Phase
        if let Some(inst) = &transaction.instruction {
            match inst.mode {
                LineMode::Quad => self.set_mode_quad_tx(),
                _ => self.set_mode_single_tx(),
            }
            self.sm.tx().stalled(); // Clear any previous stall before pushing
            self.write_byte(inst.value as u8);
            self.wait_tx_done();
        }

        // 2. Address Phase
        if let Some(addr) = &transaction.address {
            match addr.mode {
                LineMode::Quad => self.set_mode_quad_tx(),
                _ => self.set_mode_single_tx(),
            }
            self.sm.tx().stalled(); // Clear stall before pushing
            let val = addr.value;
            let len = addr.bytes_len as usize;
            for i in (0..len).rev() {
                let b = ((val >> (i * 8)) & 0xFF) as u8;
                self.write_byte(b);
            }
            self.wait_tx_done();
        }

        // 3. Data Phase
        if !data.is_empty() {
            match transaction.data_mode {
                LineMode::Quad => {
                    self.set_mode_quad_tx();
                    self.sm.tx().stalled();

                    let (_, tx) = self.sm.rx_tx();
                    let mut tx_ch = self.tx_dma.reborrow();

                    tx.dma_push(&mut tx_ch, data, false).await;

                    self.wait_tx_done();
                }
                LineMode::Single | LineMode::None => {
                    self.set_mode_single_tx();
                    self.sm.tx().stalled();

                    let (_, tx) = self.sm.rx_tx();
                    let mut tx_ch = self.tx_dma.reborrow();

                    tx.dma_push(&mut tx_ch, data, false).await;

                    self.wait_tx_done();
                }
                _ => {}
            }
        }

        let _ = self.cs.set_high();
        Ok(())
    }

    async fn read(
        &mut self,
        transaction: &QspiTransaction,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        let _ = self.cs.set_low();

        // 1. Instruction Phase
        if let Some(inst) = &transaction.instruction {
            match inst.mode {
                LineMode::Quad => self.set_mode_quad_tx(),
                _ => self.set_mode_single_tx(),
            }
            self.sm.tx().stalled();
            self.write_byte(inst.value as u8);
            self.wait_tx_done();
        }

        // 2. Address Phase
        if let Some(addr) = &transaction.address {
            match addr.mode {
                LineMode::Quad => self.set_mode_quad_tx(),
                _ => self.set_mode_single_tx(),
            }
            self.sm.tx().stalled();
            let val = addr.value;
            let len = addr.bytes_len as usize;
            for i in (0..len).rev() {
                let b = ((val >> (i * 8)) & 0xFF) as u8;
                self.write_byte(b);
            }
            self.wait_tx_done();
        }

        // 3. Read Data Phase (Single Line)
        if !buffer.is_empty() {
            self.set_mode_single_rx();

            let len = buffer.len();
            let (rx, tx) = self.sm.rx_tx();
            let mut rx_ch = self.rx_dma.reborrow();
            let mut tx_ch = self.tx_dma.reborrow();

            let rx_transfer = rx.dma_pull(&mut rx_ch, buffer, false);
            let tx_transfer = tx.dma_push_zeros::<u8>(&mut tx_ch, len);
            join(tx_transfer, rx_transfer).await;
        }

        let _ = self.cs.set_high();
        Ok(())
    }
}
