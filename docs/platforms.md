# Targets & Platforms

Where the benchmarks run, how each is timed, and their current status.

| Platform            | `probe-rs` chip | Target triple               | Timing source                | Unit   | Status                             |
| ------------------- | --------------- | --------------------------- | ---------------------------- | ------ | ---------------------------------- |
| host                | n/a             | native                      | `std::time::Instant`         | ns     | running                            |
| stm32 (F405/F411)   | `STM32F405RGTx` | `thumbv7em-none-eabihf`     | DWT cycle counter            | cycles | running                            |
| rp2040 (Pico 1)     | `RP2040`        | `thumbv6m-none-eabi`        | SysTick 24-bit down-counter¹ | cycles | running                            |
| rp2350-arm (Pico 2) | `RP235x`        | `thumbv8m.main-none-eabihf` | DWT cycle counter            | cycles | running                            |
| esp32s3             | `esp32s3`       | `xtensa-esp32s3-none-elf`   | Xtensa `CCOUNT` register²    | cycles | running                            |

¹ RP2040 has no DWT; times via SysTick's 24-bit down-counter (core cycles, caps a single
measurement at ~16.7M cycles). The 64-bit `TIMER` is the fallback for longer or
absolute-time measurements. RP2350 does have DWT and times like STM32; boots via an
`IMAGE_DEF` block, not boot2; its `-eabihf` target gives it hardware float. Future axes
(not implemented): DCP-based f64 (`dcp-fast-f64`, off by default), and Arm vs RISC-V via
its Hazard3 cores.

² ESP32-S3 is Xtensa, not ARM: needs the `espup`-installed `esp` toolchain, not
`rustup target add`. Not a `benchmarks/` workspace member (`esp-hal` and `rp235x-hal` both set
`links = "riscv-rt"` at incompatible versions, which Cargo forbids graph-wide even though each
is target-gated to a chip family the other doesn't build), so it has its own `Cargo.lock`. Skips `crazyflie-fw`
(ARM-only libm cross-link). Its USB Serial/JTAG Controller enumerates as a probe-rs probe
directly, no external wiring needed. Clean-exit uses the `semihosting` crate's
`openocd-semihosting` feature (`cortex-m-semihosting` is ARM-only).
