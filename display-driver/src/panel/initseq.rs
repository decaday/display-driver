use embedded_hal_async::delay::DelayNs;

use crate::DisplayBus;

/// A step in the initialization sequence.
#[derive(Clone, Copy)]
pub enum InitStep<'a> {
    /// Single byte command.
    SingleCommand(u8),
    /// Command with parameters.
    CommandWithParams((u8, &'a [u8])),
    /// Delay in milliseconds.
    DelayMs(u8),
    /// No Operation. Useful for placeholders or conditional logic.
    Nop,
    /// Nested sequence.
    /// NOTE: Only supports one level of nesting (cannot nest a Nested inside a Nested)
    /// to avoid recursion issues in async no_std environments.
    Nested(&'a [InitStep<'a>]),
}

impl InitStep<'static> {
    pub const fn param_option_to_step<const N: usize>(
        cmd: u8,
        params: Option<&'static [u8; N]>,
    ) -> InitStep<'static> {
        match params {
            Some(params) => InitStep::CommandWithParams((cmd, params)),
            None => InitStep::Nop,
        }
    }
}

/// Helper to execute initialization steps.
pub struct SequencedInit<'a, D: DelayNs, B: DisplayBus, I: Iterator<Item = InitStep<'a>>> {
    steps: I,
    delay: &'a mut D,
    display_bus: &'a mut B,
}

impl<'a, D: DelayNs, B: DisplayBus, I: Iterator<Item = InitStep<'a>>> SequencedInit<'a, D, B, I> {
    /// Creates a new SequencedInit instance.
    pub fn new(steps: I, delay: &'a mut D, display_bus: &'a mut B) -> Self {
        Self {
            steps,
            delay,
            display_bus,
        }
    }

    /// Helper function to execute a single atomic step.
    /// Does not handle recursion; ignores Nested variants if passed directly.
    async fn exec_atomic_step(&mut self, step: InitStep<'a>) -> Result<(), B::Error> {
        match step {
            InitStep::SingleCommand(cmd) => self.display_bus.write_cmds(&[cmd]).await,
            InitStep::CommandWithParams((cmd, data)) => {
                self.display_bus.write_cmd_with_params(&[cmd], data).await
            }
            InitStep::DelayMs(ms) => {
                self.delay.delay_ms(ms as u32).await;
                Ok(())
            }
            InitStep::Nop => Ok(()),
            InitStep::Nested(_) => {
                panic!("We only support 1 level in InitStep.");
            }
        }
    }

    /// Executes the initialization sequence.
    pub async fn sequenced_init(&mut self) -> Result<(), B::Error> {
        while let Some(step) = self.steps.next() {
            match step {
                // If the step is a Nested sequence, unroll it here (1 level deep)
                InitStep::Nested(sub_steps) => {
                    for sub_step in sub_steps.iter() {
                        // We use *sub_step because we derived Copy
                        self.exec_atomic_step(*sub_step).await?;
                    }
                }
                // Otherwise, execute the step directly
                _ => self.exec_atomic_step(step).await?,
            }
        }

        Ok(())
    }
}

/// Convenience function to run an initialization sequence.
pub async fn sequenced_init<'a, D: DelayNs, B: DisplayBus, I: Iterator<Item = InitStep<'a>>>(
    steps: I,
    delay: &'a mut D,
    display_bus: &'a mut B,
) -> Result<(), B::Error> {
    SequencedInit::new(steps, delay, display_bus)
        .sequenced_init()
        .await
}
