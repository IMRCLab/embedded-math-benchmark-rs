# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Embedded microbenchmarks comparing **C vs Rust** math libraries on compute-constrained
robotic hardware (STM32, RP2040/RP2350, ESP32-S3, nRF52840). Goal: quantify the cost and
numerical accuracy of robotic workloads (controllers, state estimators, linear algebra) per
language, library, chip and build profile. See [README.md](README.md).

## Repo layout

- `benchmarks/` — Cargo workspace (crates below)
- `docs/` — source of truth, see below
- `viz/` — uv project (`pyproject.toml`/`uv.lock`), renders `results.csv` into a
  per-platform PDF report; entry point `plot.py`, see [docs/benchmark-viz.md](docs/benchmark-viz.md)
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

- [README.md](README.md) — overview and documentation index, start here
- [docs/adding_a_benchmark.md](docs/adding_a_benchmark.md) — full recipe for a new task
- [docs/adding_a_platform.md](docs/adding_a_platform.md) — checklist for a new hardware target
- [docs/task-categories.md](docs/task-categories.md) — what each task measures & which comparisons it supports
- [docs/accuracy_evaluation.md](docs/accuracy_evaluation.md) — scientific methodology & ULP evaluation
- [docs/io-format.md](docs/io-format.md) — `inputs.json` in, `BENCH ` CSV out
- [docs/running-benchmarks.md](docs/running-benchmarks.md) — build/flash/run per platform
- [docs/platforms.md](docs/platforms.md) — chips, target triples, timing sources, status
- [docs/hil-setup.md](docs/hil-setup.md) — probe + GitHub Actions HIL CI provisioning
- [docs/build-profiles.md](docs/build-profiles.md) — the four profiles, and `xlto` internals
- [docs/c-suites.md](docs/c-suites.md) — `crazyflie-fw` / `cmsis-dsp` provenance and naming

## Architecture (the one thing to internalize)

Everything is **JSON-driven and generated at compile time** so `no_std` firmware never
parses JSON at runtime. `benchmarks/inputs.json` is read by proc-macros
(`mrs-benchmark-macros`) which emit static input arrays and the per-platform runner. A task
is defined once and runs on every platform/library from that single config.

A benchmark = **Task** (identifier + input/output shape) × **Library** implementation
(`glam`, `nalgebra`, `micromath`, `libm` in Rust; `crazyflie-fw`, `cmsis-dsp` in C). Each
library impl has three phases; **only `execute` is timed** — `prepare` (convert to lib types)
and `finalize` (convert back) are not.

Adding a task touches five coordinated places — the identifier string must match in all:

1. Input struct + `#[benchmark_input("Name")]` in `mrs-benchmark-core/src/inputs.rs`
2. `BenchmarkTask` impl in `mrs-benchmark-core/src/tasks.rs`
3. `RawTaskImplementation` + `export_tasks!` in `mrs-benchmark-core/src/suites/<lib>.rs`
4. A `BenchmarkTask` subclass in `tools/inputs_generator/tasks.py`, then `cargo make update`
5. Which regenerates `benchmarks/inputs.json`; never hand-edit that file

You never touch the platform `main.rs` files — the macro weaves tasks in. Full walkthrough:
[docs/adding_a_benchmark.md](docs/adding_a_benchmark.md).

## Crate map (`benchmarks/` Cargo workspace)

- `mrs-benchmark-core` — `no_std`. Traits, task/input registry, per-library suites. The heart.
- `mrs-benchmark-macros` — proc-macros reading `inputs.json` at build time.
- `mrs-benchmark-host` — native runner, times in `ns`.
- `mrs-benchmark-stm32` / `mrs-benchmark-rp2040` / `mrs-benchmark-rp2350` — `no_std` firmware, time in `cycles`, RTT out. (rp2350 = Pico 2, Cortex-M33: DWT timing like stm32, but boots via an `IMAGE_DEF` block, not boot2.)
- `mrs-benchmark-nrf52840` — `no_std` firmware, Cortex-M4F. Same target triple, DWT timing, and both C suites as stm32 with no target-specific plumbing; `nrf52840-hal` only selects the HFXO crystal (core is fixed at 64 MHz). See [docs/platforms.md](docs/platforms.md#nrf52840).
- `mrs-benchmark-esp32s3` — `no_std` firmware, Xtensa (not ARM), CCOUNT register timing, RTT out via native USB JTAG. Needs the `espup`-installed `esp` Rust toolchain, not plain `rustup target add`. **Not a `benchmarks/` workspace member**: `esp-hal`'s `riscv-rt` version conflicts with `rp235x-hal`'s, so it has its own standalone `Cargo.lock` (still built from its own crate dir like the others). Skips the `crazyflie-fw` library (that suite's C build only cross-links for ARM).
- `mrs-benchmark-crazyflie-sys` — the C side, ARM-only. `build.rs` uses `cc` + `bindgen` to
  compile the Crazyflie firmware math and a subset of CMSIS-DSP, feeding both the `crazyflie-fw`
  and `cmsis-dsp` suites. See [docs/c-suites.md](docs/c-suites.md).
- `mrs-benchmark-collect` — greps `BENCH ` rows from logs → one merged `results.csv`.

## Commands

Prefer `cargo-make` (`cargo install cargo-make`), run from repo root:

```bash
cargo make inputs                           # generate inputs.json via tasks.py
cargo make generate-ref                     # compute f64 ground truth reference_results.json
cargo make update                           # update inputs.json and reference_results.json together
cargo make bench-host                       # native, ns
cargo make bench-stm32                      # STM32F405 over probe-rs, cycles
cargo make bench-nrf52840                   # nRF52840 over probe-rs, cycles
cargo make bench-rp2040                     # RP2040 over probe-rs, cycles
cargo make bench-esp32s3                    # ESP32-S3 over probe-rs, cycles
cargo make bench-host | cargo make collect -- -o results.csv
cargo make eval-accuracy                    # evaluate ULP distance & relative error vs f64 ref
cargo make report                           # generate report.pdf with Pareto trade-off pages
cargo make paper-figures                    # render docs/draft/images/*.pdf for the paper (viz/paper_figures.py)
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

- **Code comments: rare and one line.** Only comment what the code can't say itself: a non-obvious
  _why_, a hidden invariant, a workaround. One line each, plain ASCII, no multi-line blocks or
  banners. Background and rationale go in the commit message or `docs/`, not above the code.
- **No em dashes, en dashes or spaced double hyphens** in prose, docs, commit messages or
  comments. Hyphens in compound words are fine.
- **Build firmware from its own crate dir** (`cd benchmarks/mrs-benchmark-stm32`) so the
  crate-local `.cargo/config.toml` selects the right thumb target and linker script. Building from
  the workspace root gets the wrong target.
- **Crate dir name == package/binary name.** CI and cargo-make derive paths from this; keep it.
- **Output = `BENCH `-prefixed CSV rows** on stdout/RTT. `collect` treats rows as opaque text, so
  the column format can evolve without touching the tool. Never break the `BENCH ` prefix or
  `CSV_HEADER` in [core `lib.rs`](benchmarks/mrs-benchmark-core/src/lib.rs).
- **`inputs.json` is baked at build time** and is generated from `tools/inputs_generator/tasks.py`.
  Changing it requires a rebuild; there is no runtime config on firmware.
- **Four build profiles ship** (`release`, `lto`, `size`, `xlto`), `panic = "abort"` in every one.
  `xlto` is ARM hard-float only and is not a default upgrade over `lto`. See
  [docs/build-profiles.md](docs/build-profiles.md).
- **The valid profile for a C-vs-Rust number depends on the task's ABI class**: free-ABI at `lto`,
  `mat33` by-value and pointer-ABI at `xlto`. At matched ABI and profile the two languages land
  within ~25%, both directions. Cite the profile with every cross-language claim. Table in
  [docs/task-categories.md](docs/task-categories.md#which-profile-makes-a-c-vs-rust-number-valid).
- **A task touching a transcendental measures the linked math provider, not the language.** The
  Rust `libm` crate loses to newlib on `sqrt` (1.8x) and `sincos` (10x) and wins on `atan2`, `exp`
  and `ln`. See
  [docs/task-categories.md](docs/task-categories.md#which-transcendental-provider-you-link-decides-the-cost).
- **Never quote `crazyflie-fw` composite ULP as accuracy.** The f64 reference follows the Rust
  operation order, so `EkfStep`'s 789,913 ULP is divergence from a different filter, not error.
  See [docs/accuracy_evaluation.md](docs/accuracy_evaluation.md#the-composite-reference-is-not-neutral).
- **On RP2040 the C suites' transcendental calls resolve to `compiler_builtins`, not newlib.**
  Read those rows as two software implementations. See
  [docs/platforms.md](docs/platforms.md#rp2040-pico-1).
- **A suite that can't build on every platform is Cargo-feature-gated.** Both C suites sit behind
  `mrs-benchmark-core`'s `crazyflie` feature; a platform that can't build them sets
  `default-features = false` on that dep. `suite_feature_gate()` in `mrs-benchmark-macros` gates
  the generated call sites to match, since `inputs.json` is shared across platforms. See
  [docs/c-suites.md](docs/c-suites.md).
- **`library_versions.json` is generated by `collect`**, not hand-maintained, and is gitignored.

Check [docs/platforms.md](docs/platforms.md) for live per-platform status before assuming a
target works.
