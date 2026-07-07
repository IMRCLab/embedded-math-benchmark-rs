# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Embedded microbenchmarks comparing **C vs Rust** performance and scheduling on
compute-constrained robotic hardware (STM32, RP2040/RP2350). Long-term goal: quantify
robotic workloads (controllers, state estimators, math), evaluate QP solvers, and compare
scheduling (FreeRTOS vs Embassy). See [README.md](README.md) and
[docs/project_requirements.md](docs/project_requirements.md).

**Current focus is milestone 1 only: benchmarking embedded math libraries** (glam,
nalgebra, micromath, and eventually C equivalents) across platforms. QP solvers and
RTOS/Embassy scheduling comparisons are later milestones — don't prioritize them or assume
they're in scope unless the user brings them up.

## Docs are the source of truth

`docs/` is maintained and current — read it before changing behavior, and update it in the
same change. This file is orientation only; don't duplicate doc detail here (it drifts).

**Keep both current.** When a task teaches you something a future session would benefit
from knowing — a new convention, a gotcha, a changed workflow, a new crate — update the
relevant `docs/*.md` file and/or this file as part of that same task, not as an afterthought.
If unsure whether something belongs in docs (durable, project-wide) vs this file
(orientation) vs memory (cross-project, about you/the user), default to updating docs.

- [docs/benchmark.md](docs/benchmark.md) — documentation index, start here
- [docs/adding_a_benchmark.md](docs/adding_a_benchmark.md) — full recipe for a new task
- [docs/io-format.md](docs/io-format.md) / [docs/inputs_configuration.md](docs/inputs_configuration.md) — `inputs.json` in, `BENCH ` CSV out
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
- `mrs-benchmark-stm32` / `mrs-benchmark-rp2040` — `no_std` firmware, time in `cycles`, RTT out. (`rp2350` planned.)
- `mrs-benchmark-collect` — greps `BENCH ` rows from logs → one merged `results.csv`.

## Commands

Prefer `cargo-make` (`cargo install cargo-make`), run from repo root:

```bash
cargo make bench-host                       # native, ns
cargo make bench-stm                        # STM32F405 over probe-rs, cycles
cargo make bench-pico                       # RP2040 over probe-rs, cycles
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

## State (2026-07, moves fast)

Host + RP2040 + STM32 run and benchmark on hardware in GitLab CI (RP2040 on HIL), all with
Rust math libraries. RP2350 support and C implementations (still milestone 1) are next up.
QP solvers and scheduling (FreeRTOS/Embassy) are later milestones, not current work.
Check [docs/platforms.md](docs/platforms.md) for live status before assuming a target works.
