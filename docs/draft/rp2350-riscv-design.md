# Draft: `rp2350-riscv` target

Status: **not scheduled**. The `probe-rs` flashing blocker has a verified workaround — see
[Targets & Platforms: RP2350](../platforms.md#rp2350-pico-2) for the chip behavior, the
workaround, and recovery. This doc covers only the not-yet-built crate.

## Shape: new sibling crate `mrs-benchmark-rp2350-riscv`

Not a fork of `mrs-benchmark-rp2350`: a new crate dir, same pattern as every other
platform (own `.cargo/config.toml`, own `main.rs`, own CI jobs).

## Decisions

- **Workspace membership**: joins `benchmarks/` (unlike ESP32-S3). `rp235x-hal`'s
  `riscv-rt` dep is target-cfg-gated internally, so it doesn't clash with the ARM crate's
  `rp235x-hal` in one `Cargo.lock` — ESP32-S3's standalone-lockfile problem was two
  different HALs wanting different `riscv-rt` versions, which doesn't apply here.
- **`crazyflie-fw`**: disabled (`default-features = false` on the `mrs-benchmark-core` dep),
  same as ESP32-S3, since the Dockerfile only ships an ARM C cross-compiler.
- **CI**: `build-rp2350-riscv` and `clippy-rp2350-riscv` only, no `run-rp2350-riscv` (HIL)
  job until the recovery procedure is scripted into CI (see
  [HIL Setup](../hil-setup.md#power-cycling-a-board)). When it lands, `select-probe.sh`
  needs one new case: `probe-rs info`'s ROM-table scan always prints `RP235x CoreSight ROM`
  regardless of chip variant, so the default `signature="$chip"` fallback won't match
  `CHIP=RP235x_riscv` — add `RP235x_riscv) signature='RP235x' ;;`, reusing the ARM job's
  signature.
- **Reporting**: `viz/config.py` needs `PLATFORM_CLOCK_HZ["rp2350-riscv"]` (both cores share
  `clk_sys`, likely 150 MHz like `rp2350-arm` — confirm once `main.rs` sets it explicitly)
  and a `PLATFORM_ORDER` entry (before `"host"`, which always sorts last).

## Open questions

- Does the HIL `gitlab-runner` probe-rs install (separate from the personal one used for the
  2026-08-12 verification) need upgrading to 0.32.0+?
- CI sequencing so a recovery step always runs between `rp2350-riscv` and `rp2350-arm` jobs
  on the same board.
