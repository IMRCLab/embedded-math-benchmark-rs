# Targets & Platforms

Where the benchmarks run, how each is timed, and their current status.

| Platform              | `probe-rs` chip | Target triple                  | Timing source                 | Unit   | Status  |
| --------------------- | --------------- | ------------------------------ | ----------------------------- | ------ | ------- |
| host                  | n/a             | native                         | `std::time::Instant`          | ns     | running |
| stm32 (F405/F411)     | `STM32F405RGTx` | `thumbv7em-none-eabihf`        | DWT cycle counter             | cycles | running |
| rp2040 (Pico 1)       | `RP2040`        | `thumbv6m-none-eabi`           | SysTick 24-bit down-counter   | cycles | running |
| rp2350-arm (Pico 2)   | `RP235x`        | `thumbv8m.main-none-eabihf`    | DWT cycle counter             | cycles | running |
| esp32s3               | `esp32s3`       | `xtensa-esp32s3-none-elf`      | Xtensa `CCOUNT` register      | cycles | running |
| rp2350-riscv (Pico 2) | `RP235x_riscv`  | `riscv32imac-unknown-none-elf` | RISC-V `mcycle`/`mcycleh` CSR | cycles | planned |
| nrf52840              | `nRF52840_xxAA` | `thumbv7em-none-eabihf`        | DWT cycle counter             | cycles | planned |

## RP2040 (Pico 1)

No DWT, unlike every other Cortex-M target here. Times via SysTick's 24-bit down-counter
(core cycles, wraps every ~16.7M cycles / ~134ms at 125MHz). TICKINT counts wraps via
interrupt, extending the range to 56 bits. Verified on real hardware 2026-08-12: 100 reps
of `MatMul9x9`/`cmsis-dsp` (~17.3-17.4M cycles) read back correctly past the wrap.

**Known gap: C suites' transcendental math calls aren't real newlib here.** `sqrtf`/`sinf`/
`cosf`/`atan2f`/`expf`/`logf` calls from `crazyflie-fw`/`cmsis-dsp` C code resolve to Rust's
own `compiler_builtins` soft-float fallback, not the real arm-none-eabi newlib. Root cause:
`rustc` always links `libcompiler_builtins.rlib` before a crate's requested `-lm`; both are
scanned lazily, so once compiler_builtins's (normally weak) symbol resolves the reference,
`libm.a`'s definition is never pulled in even though its search path is correct.
`mrs-benchmark-crazyflie-sys/build.rs` fixes this on `stm32`/`rp2350-arm` by whole-archiving
the real `libm.a` (`+whole-archive` link modifier, forcing every member in regardless of scan
order). That fix doesn't apply here: RP2040 has no FPU, so fat LTO's identical-code-folding
merges `mrs-benchmark-core`'s own `libm`-crate calls (the Rust "libm" suite) with
compiler_builtins's vendored copy of the same upstream algorithm, making that copy a
genuinely-needed strong symbol rather than an unused weak fallback -- whole-archiving the real
`libm.a` then collides with it (duplicate symbol) instead of cleanly overriding it. Affects
only those specific library calls on this one platform; matrix ops, quaternion math, and
everything not going through a bare libm-named symbol are unaffected. Not yet fixed.

## RP2350 (Pico 2)

One chip, two cores that share the same flash and boot logic: Cortex-M33 (`rp2350-arm`, running) and Hazard3 RISC-V (`rp2350-riscv`, planned).
Which one executes is decided at every power-on, by an `IMAGE_DEF`
boot-metadata block the ROM scans for in flash. `rp2350-arm` has a DWT and times like STM32.

Target for riscv is `riscv32imac-unknown-none-elf`, no FPU. It uses the same `rp235x-hal` crate as the ARM build — `rp235x_hal::entry`
and `ImageDef::secure_exe()` are architecture-aware, so application code barely changes; only
the linker script does, `rp235x_riscv.x` (vendored from `rp-hal`'s `rp235x-hal-examples`) in
place of `link.x`. Not implemented as a crate yet — [draft spec](draft/rp2350-riscv-design.md)
has the planned crate shape and CI wiring.

**Flashing it is unsupported by `probe-rs` (0.32.0) directly** — `RP235x_riscv` is
attach-only, no `download`/`run`/autodetect. Verified workaround, real hardware,
2026-08-12: flash writes don't execute target-arch code, so the _ARM_ chip id's flash
algorithm can push RISC-V bytes into flash just fine.

```
probe-rs download --chip RP235x --binary-format elf <riscv.elf>   # flash
probe-rs reset --chip RP235x                                      # re-triggers IMAGE_DEF scan, switches live core
probe-rs attach --chip RP235x_riscv <riscv.elf>                   # RTT
```

No OpenOCD or picotool/UF2 is needed for this. Getting back to ARM
afterward is _not_ symmetric, though: the Cortex-M33 AP powers down once Hazard3 is live, and
neither a plain reset nor a power cycle brings it back (the ROM re-reads the same `IMAGE_DEF`
every time). The fix is [`tools/rp2350-rescue/`](../tools/rp2350-rescue/), which pokes
RP2350's always-on Rescue DP (`RESCUE_RESTART`, CTRL bit 31 on AP `0x80000`) via `probe-rs`'s
raw DAP API — the same mechanism `probe-rs`'s own `Rp235x::reset_system` and Raspberry Pi's
OpenOCD (`rp2350-rescue.cfg`) use internally, reached directly here since their CLI/config
only exposes it from an already-working ARM session. Run the tool, then
`probe-rs reset --chip RP235x`, then a `probe-rs download --chip RP235x --binary-format elf
--speed 1000 <arm.elf>` (retry a few times — flaky right after the poke, but makes monotonic
progress; skip `--connect-under-reset`, this rig's probe reset pin isn't reliably wired).

## ESP32-S3

Xtensa, not ARM: build with the `espup`-installed `esp` toolchain, not `rustup target add`.
Not a `benchmarks/` workspace member — `esp-hal` and `rp235x-hal` pin incompatible
`riscv-rt` versions, so it keeps its own `Cargo.lock`. Skips `crazyflie-fw` (ARM-only libm
cross-link). Its USB Serial/JTAG Controller enumerates directly as a probe-rs probe, no
external wiring needed — see [HIL Setup](hil-setup.md#probes).

## nRF52840 (planned)

Not yet physically on hand. Cortex-M4F, same core family and target triple as STM32F405.
See [draft spec](draft/nrf52840-design.md).
