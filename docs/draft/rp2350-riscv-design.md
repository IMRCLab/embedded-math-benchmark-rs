# Draft: `rp2350-riscv` target

Status: **spec only, not scheduled**. RP2350's Hazard3 RISC-V cores are reachable on the
lab Pico 2, but `probe-rs` can't flash or run a RISC-V binary on this chip yet (see
Blocker). `writing-plans` is intentionally not invoked yet.

## Why

Same silicon as `rp2350-arm`, different core (Hazard3 RISC-V vs Cortex-M33). It isolates
ISA/ABI effects (soft-float vs hardware FPU) from clock and vendor effects: no other pair
of targets in this project holds the chip constant like this. Useful for milestone 1's
math library comparisons.

## Shape: new sibling crate `mrs-benchmark-rp2350-riscv`

Not a fork of `mrs-benchmark-rp2350`: a new crate dir, same pattern as every other
platform (own `.cargo/config.toml`, own `main.rs`, own CI jobs). Keep the existing ARM
crate's name as-is (`docs/platforms.md` already reserves `rp2350-arm`/`rp2350-riscv` as the
id split).

| | `mrs-benchmark-rp2350` (ARM) | `mrs-benchmark-rp2350-riscv` |
|---|---|---|
| Target | `thumbv8m.main-none-eabihf` | `riscv32imac-unknown-none-elf` (stable, no custom toolchain) |
| Entry macro | `cortex_m_rt::entry` | `rp235x_hal::entry` (same HAL crate, arch-aware) |
| Linker script | `link.x` (from `cortex-m-rt`) | `rp235x_riscv.x`, vendored like `memory.x` today |
| Timing source | DWT cycle counter (32-bit) | RISC-V `mcycle`/`mcycleh` CSR (native 64-bit) |
| FPU | Yes, hard-float | None: Hazard3 has no FPU, soft-float |
| Platform id | `"rp2350-arm"` | `"rp2350-riscv"` |

## Decisions

- **Workspace membership**: joins `benchmarks/` (unlike ESP32-S3). `rp235x-hal`'s
  `riscv-rt` dep is target-cfg-gated internally, so it doesn't clash with the ARM crate's
  `rp235x-hal` in one `Cargo.lock`. ESP32-S3's standalone-lockfile problem was two
  different HALs (`esp-hal` vs `rp235x-hal`) wanting different `riscv-rt` versions; that
  doesn't apply here.
- **`crazyflie-fw`**: disabled (`default-features = false` on the `mrs-benchmark-core` dep),
  same as ESP32-S3, since the Dockerfile only ships an ARM C cross-compiler.
- **CI**: `build-rp2350-riscv` and `clippy-rp2350-riscv` only. No `run-rp2350-riscv` (HIL)
  job until flashing works (see Blocker). No other CI restructuring: different crate dir,
  target triple, and `BIN` name already namespace it from `rp2350-arm`, the same way
  `rp2040` and `rp2350-arm` coexist today. When `run-rp2350-riscv` does land, it needs one
  new `select-probe.sh` case arm: `probe-rs info`'s ROM-table scan is chip-variant-agnostic
  and always prints `RP235x CoreSight ROM` for this board, so the default
  `signature="$chip"` fallback won't match `CHIP=RP235x_riscv`. Add
  `RP235x_riscv) signature='RP235x' ;;`, reusing the ARM job's signature.
- **Reporting**: `viz/config.py` needs `PLATFORM_CLOCK_HZ["rp2350-riscv"]` (both cores share
  `clk_sys`, so likely 150 MHz like `rp2350-arm`; confirm once `main.rs` sets it explicitly)
  and a `PLATFORM_ORDER` entry (before `"host"`, which always sorts last). See
  `docs/adding_a_platform.md`.

## Blocker: `probe-rs` can't flash or run this chip's RISC-V cores yet

`probe-rs 0.32.0` added an `RP235x_riscv` chip variant (two RISC-V cores, `rv0`/`rv1`), but
it's attach-only: flashing, `probe-rs run`, and CPU-type autodetection aren't implemented
(per [probe-rs#3952](https://github.com/probe-rs/probe-rs/pull/3952)). So
`probe-rs run --chip RP235x_riscv <elf>` doesn't exist today. Confirmed against a
`probe-rs 0.32.0` install under a personal lab account; the HIL `gitlab-runner` account
runs its own separate `probe-rs` install (see [HIL Setup](../hil-setup.md#hil-runner)) and
hasn't been upgraded, so it'd need its own provisioning pass before `run-rp2350-riscv`
could work in CI.

## Open questions (resolve during implementation, not now)

- Flashing recipe once upstream matures. One option: flash the RISC-V ELF via the existing
  `RP235x` (ARM) chip's flash algorithm (flash writes don't execute target-arch code),
  reset, then `probe-rs attach --chip RP235x_riscv` for RTT. Unverified, needs real
  hardware.
- Whether `attach` (already available) is enough for a manual/local check before `run`
  lands.
- Exact `rp235x_riscv.x` linker script contents to vendor (from `rp-hal`'s
  `rp235x-hal-examples`).
