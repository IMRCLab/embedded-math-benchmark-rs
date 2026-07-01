# Targets & Platforms

Where the benchmarks run, how each is timed, and their current status.

| Platform | `probe-rs` chip | Target triple | Timing source | Unit | Status |
|----------|-----------------|---------------|---------------|------|--------|
| host | n/a | native | `std::time::Instant` | ns | running |
| stm32 (F405) | `STM32F405RGTx` | `thumbv7em-none-eabihf` | DWT cycle counter | cycles | software only, not on hardware yet |
| rp2040 (Pico 1) | `RP2040` | `thumbv6m-none-eabi` | SysTick or 1 µs timer¹ | cycles / µs | planned |
| rp2350-arm (Pico 2) | `RP235x` | `thumbv8m.main-none-eabihf` | DWT cycle counter | cycles | planned |

¹ The RP2040's Cortex-M0+ has no DWT cycle counter, so it uses SysTick or the 1 µs timer.
RP2350 can alternatively boot its RISC-V (Hazard3) cores (`RP235x_riscv`), a possible
future comparison axis.
