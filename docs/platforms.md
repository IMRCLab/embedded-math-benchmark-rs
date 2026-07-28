# Targets & Platforms

Where the benchmarks run, how each is timed, and their current status.

| Platform            | `probe-rs` chip  | Target triple                  | Timing source                | Unit   | Status  |
| ------------------- | ---------------- | ------------------------------- | ----------------------------- | ------ | ------- |
| host                | n/a              | native                          | `std::time::Instant`         | ns     | running |
| stm32 (F405/F411)   | `STM32F405RGTx`  | `thumbv7em-none-eabihf`         | DWT cycle counter            | cycles | running |
| rp2040 (Pico 1)     | `RP2040`         | `thumbv6m-none-eabi`            | SysTick 24-bit down-counter¹ | cycles | running |
| rp2350-arm (Pico 2) | `RP235x`         | `thumbv8m.main-none-eabihf`     | DWT cycle counter¹           | cycles | running |
| esp32s3             | `esp32s3`        | `xtensa-esp32s3-none-elf`       | Xtensa `CCOUNT` register²    | cycles | running |
| rp2350-riscv (Pico 2) | `RP235x_riscv`³ | `riscv32imac-unknown-none-elf` | RISC-V `mcycle`/`mcycleh` CSR | cycles | planned |
| nrf52840            | `nRF52840_xxAA`⁴ | `thumbv7em-none-eabihf`         | DWT cycle counter            | cycles | planned |

¹ RP2040 has no DWT; times via SysTick's 24-bit down-counter (core cycles, caps a single
measurement at ~16.7M cycles), with the 64-bit `TIMER` as fallback for longer spans. RP2350
has DWT and times like STM32; boots via an `IMAGE_DEF` block, not boot2. Its platform id is
`rp2350-arm`; the suffix leaves room for a future RISC-V (Hazard3) build of the same chip.

² ESP32-S3 is Xtensa, not ARM: build with the `espup`-installed `esp` toolchain, not
`rustup target add`. Not a `benchmarks/` workspace member (`esp-hal` and `rp235x-hal` pin
incompatible `riscv-rt` versions), so it keeps its own `Cargo.lock`. Skips `crazyflie-fw`
(ARM-only libm cross-link). Its USB Serial/JTAG Controller enumerates directly as a
probe-rs probe, no external wiring needed. See [HIL Setup](hil-setup.md#probes).

³ Same silicon as `rp2350-arm`, Hazard3 RISC-V core instead of Cortex-M33 (no FPU,
soft-float). Blocked on `probe-rs` flashing support for this chip variant (attach-only as
of 0.32.0). See [draft spec](draft/rp2350-riscv-design.md).

⁴ Planned, not yet physically on hand. Cortex-M4F, same core family and target triple as
STM32F405. See [draft spec](draft/nrf52840-design.md).
