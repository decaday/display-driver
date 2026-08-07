# STM32H7B0 QSPI Example

Due to a missing `OctoDma` trait mapping for the MDMA FIFO request in `embassy-stm32` version 0.6.0, this example relies on a patched/git version of `embassy-stm32` to support Async QSPI with MDMA.

Once a new release of `embassy-stm32` (newer than 0.6.0) is published that includes this fix, this example directory will be merged back into the standard `stm32h7b0` examples directory.
