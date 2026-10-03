use std::{
    convert::Infallible,
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

use display_driver::{
    bus::ErrorType, ColorFormat, DisplayDriver, DisplayError, LCDResetOption, Orientation,
    SimpleDisplayBus,
};
use display_driver_st7789::{
    spec::generic::{Generic240x320P3Type1, Generic240x320Type1},
    St7789, St7789Spec,
};
use embedded_hal::digital::{ErrorType as PinErrorType, OutputPin};
use embedded_hal_async::delay::DelayNs;

#[derive(Debug)]
struct Command {
    opcode: u8,
    params: Vec<u8>,
}

#[derive(Debug)]
enum BusError {
    Injected(u8),
}

struct RecordingBus<'a> {
    commands: &'a mut Vec<Command>,
    fail_on: Option<u8>,
}

impl ErrorType for RecordingBus<'_> {
    type Error = BusError;
}

impl SimpleDisplayBus for RecordingBus<'_> {
    async fn write_cmds(&mut self, cmds: &[u8]) -> Result<(), Self::Error> {
        for &opcode in cmds {
            self.commands.push(Command {
                opcode,
                params: Vec::new(),
            });
            if self.fail_on == Some(opcode) {
                return Err(BusError::Injected(opcode));
            }
        }
        Ok(())
    }

    async fn write_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.commands
            .last_mut()
            .expect("data must follow a command")
            .params
            .extend_from_slice(data);
        Ok(())
    }
}

struct ResetPin;

impl PinErrorType for ResetPin {
    type Error = Infallible;
}

impl OutputPin for ResetPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct ImmediateDelay;

impl DelayNs for ImmediateDelay {
    async fn delay_ns(&mut self, _ns: u32) {}
}

fn run_immediate<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("recording bus and delay must complete immediately"),
    }
}

fn initialize<Spec: St7789Spec>(
    commands: &mut Vec<Command>,
    fail_on: Option<u8>,
) -> Result<(), DisplayError<BusError>> {
    let bus = RecordingBus { commands, fail_on };
    let panel = St7789::<Spec, _, _>::new(LCDResetOption::new_pin(ResetPin));
    run_immediate(
        DisplayDriver::builder(bus, panel)
            .with_color_format(ColorFormat::RGB565)
            .with_orientation(Orientation::Deg180)
            .init(&mut ImmediateDelay),
    )
    .map(|_| ())
}

fn single_command(commands: &[Command], opcode: u8) -> (usize, &Command) {
    let mut matches = commands
        .iter()
        .enumerate()
        .filter(|(_, command)| command.opcode == opcode);
    let command = matches
        .next()
        .unwrap_or_else(|| panic!("missing command {opcode:#04x}"));
    assert!(matches.next().is_none(), "duplicate command {opcode:#04x}");
    command
}

#[test]
fn p3_initializes_with_vcom_offset_and_portrait_rgb565() {
    let mut commands = Vec::new();
    initialize::<Generic240x320P3Type1>(&mut commands, None).unwrap();

    assert!(commands.iter().all(|command| command.opcode != 0xC4));
    let (c5_index, c5) = single_command(&commands, 0xC5);
    assert_eq!(c5.params, [0x20]);
    let c3_index = commands
        .iter()
        .position(|command| command.opcode == 0xC3)
        .expect("VRHS must precede VCMOFSET");
    let c6_index = commands
        .iter()
        .position(|command| command.opcode == 0xC6)
        .expect("FRCTRL2 must follow VCMOFSET");
    assert!(c3_index < c5_index && c5_index < c6_index);

    let c2 = commands
        .iter()
        .find(|command| command.opcode == 0xC2)
        .expect("VDVVRHEN must be sent");
    assert_eq!(c2.params, [0x01, 0xFF]);
    let colmod = commands
        .iter()
        .rfind(|command| command.opcode == 0x3A)
        .expect("COLMOD must be sent");
    assert_eq!(colmod.params, [0x55]);
    let madctl = commands
        .iter()
        .rfind(|command| command.opcode == 0x36)
        .expect("MADCTL must be sent");
    assert_eq!(madctl.params, [0xC0]);
    let inversion = commands
        .iter()
        .rfind(|command| matches!(command.opcode, 0x20 | 0x21));
    assert_eq!(inversion.map(|command| command.opcode), Some(0x21));
}

#[test]
fn legacy_preset_keeps_vdv_without_vcom_offset() {
    let mut commands = Vec::new();
    initialize::<Generic240x320Type1>(&mut commands, None).unwrap();

    let (_, c4) = single_command(&commands, 0xC4);
    assert_eq!(c4.params, [0x20]);
    assert!(commands.iter().all(|command| command.opcode != 0xC5));
}

#[test]
fn vcom_offset_bus_error_stops_initialization_before_display_on() {
    let mut commands = Vec::new();
    let result = initialize::<Generic240x320P3Type1>(&mut commands, Some(0xC5));

    assert!(matches!(
        result,
        Err(DisplayError::BusError(BusError::Injected(0xC5)))
    ));
    assert!(commands.iter().all(|command| command.opcode != 0x29));
}
