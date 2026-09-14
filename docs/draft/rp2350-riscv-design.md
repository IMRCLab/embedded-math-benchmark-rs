# Draft: `rp2350-riscv` target

Status: **not scheduled**. The `probe-rs` flashing blocker has a verified workaround; see
[`tools/rp2350-rescue/`](../../tools/rp2350-rescue/README.md) for flashing and recovery, and
[Targets & Platforms](../platforms.md#rp2350-pico-2) for the chip behavior. This doc covers only
the not-yet-built crate.

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
- **CI**: `build-rp2350-riscv` job (calling `build-target.yml`) and a `clippy-rp2350-riscv`
  matrix row on `check`, no `run-firmware-rp2350-riscv` (HIL) job until the recovery procedure
  is scripted into CI (see [HIL Setup](../hil-setup.md#power-cycling-a-board)). When it lands,
  the run job reuses `HIL_PROBE_RP2350` as its `probe_var`: same physical probe, only `chip`
  differs (`RP235x_riscv`).
- **Reporting**: `viz/config.py` needs `PLATFORM_CLOCK_HZ["rp2350-riscv"]` (both cores share
  `clk_sys`, likely 150 MHz like `rp2350-arm` — confirm once `main.rs` sets it explicitly)
  and a `PLATFORM_ORDER` entry (before `"host"`, which always sorts last).

## Recovery in CI

- **Scope**: a shared `recover-rp2350` job runs unconditionally before _both_
  `run-firmware-rp2350` and `run-firmware-rp2350-riscv`, every run — not just ahead of the
  riscv leg. The stuck-board failure mode isn't riscv-specific (it already happened once
  during manual testing), so making the board's starting state irrelevant is worth the few
  extra seconds on every rp2350 job.
- **Rescue binary**: built in CI (`build-rp2350-rescue`, on the `build` runner, native
  compile, `/persist` cargo cache) and shipped as an artifact, same pattern as the firmware
  ELFs. Keeps the flash runners artifact-only, same as every other `run-firmware-*` job (no
  checkout, workspace persists between runs).
- **`recover-rp2350` job** (flash runner pool): run the rescue binary against the
  `HIL_PROBE_RP2350` repo variable (idempotent poke) -> `probe-rs reset --chip RP235x` ->
  retry loop (~5x, 2s apart) of a lightweight `probe-rs info --chip RP235x` until it
  succeeds. All the documented post-poke flakiness gets absorbed here, once —
  `run-firmware-rp2350` and `run-firmware-rp2350-riscv` need no retry logic of their own once
  this job reports success.
- **`run-firmware-rp2350` / `run-firmware-rp2350-riscv`**: both add `needs: [recover-rp2350]`.
  `run-firmware-rp2350-riscv` can't reuse `flash-target.yml`'s single `probe-rs run` step — it
  needs the documented 3-step dance instead (`download --chip RP235x` with the riscv ELF,
  `reset --chip RP235x`, `attach --chip RP235x_riscv`).
- **Failure semantics**: `recover-rp2350` stays `continue-on-error: true`, matching the
  existing flash jobs. Caveat: GitHub Actions still runs `needs:` dependents when the needed
  job fails with `continue-on-error: true` (its conclusion still reports as `success`, so it
  doesn't skip them) — a wedged board means the two run jobs still attempt and each burn up
  to the 240s per-attempt flash timeout, rather than being skipped. The run stays green
  either way; accepted as consistent with the existing pattern.
