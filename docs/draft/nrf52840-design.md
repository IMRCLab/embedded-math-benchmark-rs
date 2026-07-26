# Draft: nRF52840 bring-up

Status: planned, no hardware on hand yet. Not actionable; hold `writing-plans` until it
arrives.

## Chip facts

- Cortex-M4F, hardware FPU, same family as STM32F405. Target `thumbv7em-none-eabihf`
  (already in the CI image).
- 1 MB flash at `0x00000000`, 256 KB RAM at `0x20000000`.
- DWT cycle counter, same as STM32/RP2350.
- No onboard debugger; needs an external SWD probe wired to the debug pads.

## Crate: `mrs-benchmark-nrf52840`

- Mirrors `mrs-benchmark-stm32`: `nrf52840-hal` for clock config only (explicitly select
  the HFXO crystal for a stable 64 MHz core clock, matching STM32's explicit `sysclk`
  setup rather than reset defaults), `cortex-m`/`cortex-m-rt`/`cortex-m-semihosting`/
  `rtt-target` (`ChannelMode::BlockIfFull`, explicit buffer size), `panic-halt`.
- `memory.x`: `FLASH` 1024K at `0x00000000`, `RAM` 256K at `0x20000000`; `build.rs` copies
  it into `OUT_DIR` like STM32's.
- `.cargo/config.toml`: `target = "thumbv7em-none-eabihf"`, `-Tlink.x`.
- Joins `benchmarks/Cargo.toml` workspace `members` (not `default-members`).
- `crazyflie-fw`: expected to cross-link clean (ARM, like STM32); gate off via
  `default-features = false` if it doesn't.

## CI (once hardware is in hand)

- `check.yml`: `clippy-nrf52840`, mirrors `clippy-stm32`.
- `build.yml`: `build-nrf52840`, `BIN: mrs-benchmark-nrf52840`, `TARGET:
  thumbv7em-none-eabihf`. No `Dockerfile.ci` change needed.
- `run.yml`: `run-nrf52840`, `CHIP: <probe-rs chip id, TBD>`.
- `collect.yml`: add `run-nrf52840` to `collect-results`' `needs` (`optional: true`).
- `select-probe.sh`: new case arm once the actual probe's signature is known.
- `viz/plot.py`: `PLATFORM_CLOCK_HZ["nrf52840"] = 64e6` (confirm), `PLATFORM_ORDER` entry.
- `docs/platforms.md`: flip to `running`, fill in the real chip id.
- `docs/hil-setup.md`: note the SWD wiring and probe once known.

## Open questions

- Exact `probe-rs` chip id (expect `nRF52840_xxAA`).
- Which physical probe gets wired to it.
- Whether `crazyflie-fw` cross-links unmodified.
- Clean-exit: `cortex-m-semihosting` should work as on STM32 (both ARM); verify actual
  `probe-rs` exit handling rather than assume.
