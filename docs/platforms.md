# Targets & Platforms

Where the benchmarks run, how each is timed, and their current status.

| Platform            | `probe-rs` chip | Target triple               | Timing source                | Unit   | Status                             |
| ------------------- | --------------- | --------------------------- | ---------------------------- | ------ | ---------------------------------- |
| host                | n/a             | native                      | `std::time::Instant`         | ns     | running                            |
| stm32 (F405/F411)   | `STM32F405RGTx` | `thumbv7em-none-eabihf`     | DWT cycle counter            | cycles | running                            |
| rp2040 (Pico 1)     | `RP2040`        | `thumbv6m-none-eabi`        | SysTick 24-bit down-counter¹ | cycles | running                            |
| rp2350-arm (Pico 2) | `RP235x`        | `thumbv8m.main-none-eabihf` | DWT cycle counter            | cycles | running                            |

¹ The RP2040's Cortex-M0+ has no DWT cycle counter. The crate defaults to the SysTick
down-counter (core cycles, no clock setup needed); its 24-bit range caps a single
measurement at ~16.7M cycles. The 64-bit 1 µs `TIMER` is the alternative for longer or
absolute-time (µs) measurements. RP2350 can alternatively boot its RISC-V (Hazard3) cores
(`RP235x_riscv`), a possible future comparison axis.

The RP2350's Cortex-M33 **has** a DWT cycle counter, so its crate times exactly like the
STM32 (`DCB.enable_trace()` + `DWT.enable_cycle_counter()`), not like the RP2040. Its
`-eabihf` target enables the FPU (float args in FPU registers), making it the first
hard-float embedded data point. Unlike the RP2040 it boots not from a boot2 blob but from
an **IMAGE_DEF** metadata block the bootrom scans for in the first flash pages — provided
by an `IMAGE_DEF` static in `.start_block` (`rp235x_hal::block::ImageDef::secure_exe()`);
omit it and the image links but the ROM won't boot it.

Two future comparison axes the chip opens up (not yet implemented): **hardware f64** via
the RP2350's DCP coprocessor (`rp235x-hal`'s `dcp-fast-f64` feature — deliberately left
**off** so the baseline stays fair vs the other platforms; enable it only as a separate
labeled config), and **Arm vs RISC-V** on the same silicon via the Hazard3 cores noted
above (a `rp2350-riscv` variant).
