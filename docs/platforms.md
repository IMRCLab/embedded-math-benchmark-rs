# Targets & Platforms

Where the benchmarks run, how each is timed, and their current status.

| Platform            | `probe-rs` chip | Target triple               | Timing source                | Unit   | Status                             |
| ------------------- | --------------- | --------------------------- | ---------------------------- | ------ | ---------------------------------- |
| host                | n/a             | native                      | `std::time::Instant`         | ns     | running                            |
| stm32 (F405/F411)   | `STM32F405RGTx` | `thumbv7em-none-eabihf`     | DWT cycle counter            | cycles | running                            |
| rp2040 (Pico 1)     | `RP2040`        | `thumbv6m-none-eabi`        | SysTick 24-bit down-counter¹ | cycles | running                            |
| rp2350-arm (Pico 2) | `RP235x`        | `thumbv8m.main-none-eabihf` | DWT cycle counter            | cycles | planned                            |

¹ The RP2040's Cortex-M0+ has no DWT cycle counter. The crate defaults to the SysTick
down-counter (core cycles, no clock setup needed); its 24-bit range caps a single
measurement at ~16.7M cycles. The 64-bit 1 µs `TIMER` is the alternative for longer or
absolute-time (µs) measurements. RP2350 can alternatively boot its RISC-V (Hazard3) cores
(`RP235x_riscv`), a possible future comparison axis.
