# Targets and platforms

Where the benchmarks run, how each is timed, and their current status.

| Platform              | `probe-rs` chip | Target triple                  | Timing source                 | Unit   | Status  |
| --------------------- | --------------- | ------------------------------ | ----------------------------- | ------ | ------- |
| host                  | n/a             | native                         | `std::time::Instant`          | ns     | running |
| stm32 (F405/F411)     | `STM32F405RGTx` | `thumbv7em-none-eabihf`        | DWT cycle counter             | cycles | running |
| rp2040 (Pico 1)       | `RP2040`        | `thumbv6m-none-eabi`           | SysTick 24-bit down-counter   | cycles | running |
| rp2350-arm (Pico 2)   | `RP235x`        | `thumbv8m.main-none-eabihf`    | DWT cycle counter             | cycles | running |
| esp32s3               | `esp32s3`       | `xtensa-esp32s3-none-elf`      | Xtensa `CCOUNT` register      | cycles | running |
| rp2350-riscv (Pico 2) | `RP235x_riscv`  | `riscv32imac-unknown-none-elf` | RISC-V `mcycle`/`mcycleh` CSR | cycles | planned |
| nrf52840              | `nRF52840_xxAA` | `thumbv7em-none-eabihf`        | DWT cycle counter             | cycles | running |

Measured with rustc 1.98.0 (LLVM 22.1.8) on ARM, the espup Xtensa fork of 1.97.0-nightly on
esp32s3, `arm-none-eabi-gcc` 14.2.1 (newlib) for the C suites, and clang 19.1.7 for `xlto`.

## RP2040 (Pico 1)

No DWT, unlike every other Cortex-M target here. Times via SysTick's 24-bit down-counter (core
cycles, wrapping every ~16.7M cycles at 125 MHz); TICKINT counts the wraps by interrupt, extending
the range to 56 bits.

**Known gap: the C suites' transcendental calls are not real newlib here.** `rustc` links
`libcompiler_builtins.rlib` before a crate's requested `-lm` and both are scanned lazily, so
`compiler_builtins`' own soft-float fallback resolves `sqrtf`/`sinf`/`cosf`/`atan2f`/`expf`/`logf`
first. `mrs-benchmark-crazyflie-sys/build.rs` whole-archives the real `libm.a` on stm32 and
rp2350-arm; that fix collides with identical-code-folding here, because RP2040 has no FPU and fat
LTO merges the two copies of the same algorithm. Read transcendental rows on this platform as two
software implementations rather than newlib against Rust. Matrix and quaternion ops are unaffected.

## RP2350 (Pico 2)

One chip, two cores sharing the same flash: Cortex-M33 (`rp2350-arm`, running) and Hazard3 RISC-V
(`rp2350-riscv`, planned). Which one executes is decided at every power-on by an `IMAGE_DEF`
block the boot ROM scans for in flash. `rp2350-arm` has a DWT and times like STM32.

The RISC-V target has no FPU and reuses the same `rp235x-hal` crate: `rp235x_hal::entry` and
`ImageDef::secure_exe()` are architecture-aware, so only the linker script changes
(`rp235x_riscv.x`, vendored from `rp-hal`'s examples, in place of `link.x`). Not implemented as a
crate yet, see the [spec](draft/rp2350-riscv-design.md). Flashing it and recovering the ARM core
afterward need [`tools/rp2350-rescue/`](../tools/rp2350-rescue/README.md).

## ESP32-S3

Xtensa, not ARM: build with the `espup`-installed `esp` toolchain, not `rustup target add`. Not a
`benchmarks/` workspace member, because `esp-hal` and `rp235x-hal` pin incompatible `riscv-rt`
versions, so it keeps its own `Cargo.lock`. Skips both C suites, which only cross-link for ARM.
Its USB Serial/JTAG controller enumerates directly as a probe-rs probe, with no external wiring,
see [HIL setup](hil-setup.md#probes).

## nRF52840

Cortex-M4F, hardware FPU, same core family and target triple (`thumbv7em-none-eabihf`) as the
STM32F405, so it shares the STM32's toolchain, DWT timing, and both C suites with no
target-specific plumbing. Core clock is fixed at 64 MHz; `main.rs` selects the external HFXO
crystal as the HFCLK source so the cycle-to-ns conversion in `viz/config.py` is exact.

**DWT on nRF52 needs a debugger attached** to count, which the HIL setup always has. A
no-debugger run would read zero cycles.

Runs on hardware over a standalone J-Link (`1366:1020:000802013924`) at 4 MHz SWD. Recent
nRF52840 silicon ships with APPROTECT locked; `probe-rs` clears it with a full-erase unlock over
CTRL-AP on first contact (nothing to preserve on a bench board).
