# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Embedded microbenchmarks comparing **C vs Rust** performance and scheduling on
compute-constrained robotic hardware (STM32, RP2040/RP2350, ESP32-S3). Long-term goal: quantify
robotic workloads (controllers, state estimators, math), evaluate QP solvers, and compare
scheduling (FreeRTOS vs Embassy). See [README.md](README.md) and
[docs/project_requirements.md](docs/project_requirements.md).

**Current focus is milestone 1 only: benchmarking embedded math libraries** (glam,
nalgebra, micromath, and eventually C equivalents) across platforms. QP solvers and
RTOS/Embassy scheduling comparisons are later milestones — don't prioritize them or assume
they're in scope unless the user brings them up.

## Repo layout

- `benchmarks/` — Cargo workspace (crates below)
- `docs/` — source of truth, see below
- `viz/` — `plot.py`, renders `results.csv` into a per-platform PDF report
- `tools/` — standalone scripts, e.g. `inputs_generator/` (generates `benchmarks/inputs.json` cases)
- `slides/` — Marp slide decks for periodic project update talks

## Docs are the source of truth

`docs/` is maintained and current — read it before changing behavior, and update it in the
same change. This file is orientation only; don't duplicate doc detail here (it drifts).

**Keep both current.** When a task teaches you something a future session would benefit
from knowing — a new convention, a gotcha, a changed workflow, a new crate — update the
relevant `docs/*.md` file and/or this file as part of that same task, not as an afterthought.
If unsure whether something belongs in docs (durable, project-wide) vs this file
(orientation) vs memory (cross-project, about you/the user), default to updating docs.

**Docs style — write for skimming.** Keep docs terse and scannable: bullets and tables over
prose, one line per decision, no padded "why X over Y" justification. Always run the
`humanizer` skill over doc prose before finishing (strip em-dash overuse, rule-of-three,
restatement, AI vocabulary). Long draft prose is scaffolding, not the deliverable.

**Use auto-memory proactively.** Maintain the project auto-memory without being asked — when
you learn a durable fact about how the user works or a cross-session convention, save/update
a memory in that same task. Reserve memory for `user`/`feedback`/`reference`; project,
hardware, and status facts go in `docs/`, not memory.

- [docs/benchmark.md](docs/benchmark.md) — documentation index, start here
- [docs/adding_a_benchmark.md](docs/adding_a_benchmark.md) — full recipe for a new task
- [docs/adding_a_platform.md](docs/adding_a_platform.md) — checklist for a new hardware target
- [docs/io-format.md](docs/io-format.md) — `inputs.json` schema (generated via `tools/`) in, `BENCH ` CSV out
- [docs/running-benchmarks.md](docs/running-benchmarks.md) — build/flash/run per platform
- [docs/platforms.md](docs/platforms.md) — chips, target triples, timing sources, status
- [docs/hil-setup.md](docs/hil-setup.md) — probe + GitLab HIL CI provisioning

## Architecture (the one thing to internalize)

Everything is **JSON-driven and generated at compile time** so `no_std` firmware never
parses JSON at runtime. `benchmarks/inputs.json` is read by proc-macros
(`mrs-benchmark-macros`) which emit static input arrays and the per-platform runner. A task
is defined once and runs on every platform/library from that single config.

A benchmark = **Task** (identifier + input/output shape) × **Library** implementation
(`glam`, `nalgebra`, `micromath`, …). Each library impl has three phases; **only `execute`
is timed** — `prepare` (convert to lib types) and `finalize` (convert back) are not.

Adding a task touches four coordinated places — the identifier string must match in all:
1. Input struct + `#[benchmark_input("Name")]` in `mrs-benchmark-core/src/inputs.rs`
2. `BenchmarkTask` impl in `mrs-benchmark-core/src/tasks.rs`
3. `RawTaskImplementation` + `export_tasks!` in `mrs-benchmark-core/src/suites/<lib>.rs`
4. A case in `benchmarks/inputs.json`

You never touch the platform `main.rs` files — the macro weaves tasks in. Full walkthrough:
[docs/adding_a_benchmark.md](docs/adding_a_benchmark.md).

## Crate map (`benchmarks/` Cargo workspace)

- `mrs-benchmark-core` — `no_std`. Traits, task/input registry, per-library suites. The heart.
- `mrs-benchmark-macros` — proc-macros reading `inputs.json` at build time.
- `mrs-benchmark-host` — native runner, times in `ns`.
- `mrs-benchmark-stm32` / `mrs-benchmark-rp2040` / `mrs-benchmark-rp2350` — `no_std` firmware, time in `cycles`, RTT out. (rp2350 = Pico 2, Cortex-M33: DWT timing like stm32, but boots via an `IMAGE_DEF` block, not boot2.)
- `mrs-benchmark-esp32s3` — `no_std` firmware, Xtensa (not ARM), CCOUNT register timing, RTT out via native USB JTAG. Needs the `espup`-installed `esp` Rust toolchain, not plain `rustup target add`. **Not a `benchmarks/` workspace member**: `esp-hal`'s `riscv-rt` version conflicts with `rp235x-hal`'s, so it has its own standalone `Cargo.lock` (still built from its own crate dir like the others). Skips the `crazyflie-fw` library (that suite's C build only cross-links for ARM).
- `mrs-benchmark-collect` — greps `BENCH ` rows from logs → one merged `results.csv`.

## Commands

Prefer `cargo-make` (`cargo install cargo-make`), run from repo root:

```bash
cargo make bench-host                       # native, ns
cargo make bench-stm                        # STM32F405 over probe-rs, cycles
cargo make bench-pico                       # RP2040 over probe-rs, cycles
cargo make bench-esp32                      # ESP32-S3 over probe-rs, cycles
cargo make bench-host | cargo make collect -- -o results.csv
```

Bare `cargo build`/`test`/`clippy` at the workspace root (`benchmarks/`) act on **native
crates only** (host, core, collect). Firmware crates are excluded from `default-members`
and need an explicit target from their own crate dir, e.g.
`cd benchmarks/mrs-benchmark-stm32 && cargo clippy -- -D warnings`.

Lint/format/test mirrors CI's `check` stage, run from `benchmarks/`:

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings                                    # native crates only
cargo test                                                      # native crates only
cargo test -p mrs-benchmark-collect --test cli <test_name>      # single test
```

Tests currently only exist in `mrs-benchmark-collect` (unit tests in `src/lib.rs`, a
CLI end-to-end test in `tests/cli.rs`) — core/host/macros have none yet.

## Conventions & gotchas

- **Build firmware from its own crate dir** (`cd benchmarks/mrs-benchmark-stm32`) so the
  crate-local `.cargo/config.toml` selects the right thumb target and linker script.
  Building from the workspace root gets the wrong target.
- **Crate dir name == package/binary name.** CI and cargo-make derive paths from this; keep it.
- **Output = `BENCH `-prefixed CSV rows** on stdout/RTT. `collect` treats rows as opaque
  text (no per-column parsing), so the column format can evolve without touching the tool.
  A failed math op emits `result` as `ERROR: <reason>`. Never break the `BENCH ` prefix or
  the header in [core `lib.rs`](benchmarks/mrs-benchmark-core/src/lib.rs) (`CSV_HEADER`).
- **`inputs.json` is baked at build time.** Changing it requires a rebuild; there's no
  runtime config on firmware.
- **`panic = "abort"`** in both profiles (workspace `Cargo.toml`).
- The C side of the C-vs-Rust comparison is **not yet implemented** — current suites are all
  Rust math libraries. C slots into the same `library` CSV field when added.
- **A suite that can't build on every platform is Cargo-feature-gated, not assumed universal.**
  `crazyflie-fw` is behind `mrs-benchmark-core`'s `crazyflie` feature (default on); a platform
  that can't build it disables `default-features` on its `mrs-benchmark-core` dependency. The
  macro also needs to know: `suite_feature_gate()` in `mrs-benchmark-macros/src/lib.rs` wraps
  that library's generated call sites in a matching `#[cfg(feature = ...)]`, since inputs.json
  is shared across all platforms and the generated runner otherwise references every library
  listed anywhere in it unconditionally.

## State (2026-07, moves fast)

Host + RP2040 + RP2350 + STM32 + ESP32-S3 run and benchmark on hardware in GitLab CI
(RP2040/RP2350/ESP32-S3 on HIL), all with Rust math libraries (ESP32-S3 minus
`crazyflie-fw`). C implementations (still milestone 1) are next up. QP solvers and
scheduling (FreeRTOS/Embassy) are later milestones, not current work. Check
[docs/platforms.md](docs/platforms.md) for live status before assuming a target works.
