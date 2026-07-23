# Targets & Platforms

Where the benchmarks run, how each is timed, and their current status.

| Platform            | `probe-rs` chip | Target triple               | Timing source                | Unit   | Status  |
| ------------------- | --------------- | --------------------------- | ----------------------------- | ------ | ------- |
| host                | n/a             | native                      | `std::time::Instant`         | ns     | running |
| stm32 (F405/F411)   | `STM32F405RGTx` | `thumbv7em-none-eabihf`     | DWT cycle counter            | cycles | running |
| rp2040 (Pico 1)     | `RP2040`        | `thumbv6m-none-eabi`        | SysTick 24-bit down-counter¹ | cycles | running |
| rp2350-arm (Pico 2) | `RP235x`        | `thumbv8m.main-none-eabihf` | DWT cycle counter¹           | cycles | running |
| esp32s3             | `esp32s3`       | `xtensa-esp32s3-none-elf`   | Xtensa `CCOUNT` register²    | cycles | running |

¹ RP2040 has no DWT; times via SysTick's 24-bit down-counter (core cycles, caps a single
measurement at ~16.7M cycles), with the 64-bit `TIMER` as fallback for longer spans. RP2350
has DWT and times like STM32; boots via an `IMAGE_DEF` block, not boot2. Its platform id is
`rp2350-arm`; the suffix leaves room for a future RISC-V (Hazard3) build of the same chip.

² ESP32-S3 is Xtensa, not ARM: build with the `espup`-installed `esp` toolchain, not
`rustup target add`. Not a `benchmarks/` workspace member (`esp-hal` and `rp235x-hal` pin
incompatible `riscv-rt` versions), so it keeps its own `Cargo.lock`. Skips `crazyflie-fw`
(ARM-only libm cross-link). Its USB Serial/JTAG Controller enumerates directly as a
probe-rs probe, no external wiring needed. See [HIL Setup](hil-setup.md#probes).
