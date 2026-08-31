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

- [docs/benchmark.md](docs/benchmark.md) — documentation index, start here
- [docs/adding_a_benchmark.md](docs/adding_a_benchmark.md) — full recipe for a new task
- [docs/adding_a_platform.md](docs/adding_a_platform.md) — checklist for a new hardware target
- [docs/task-categories.md](docs/task-categories.md) — what each task measures & which comparisons it supports
- [docs/accuracy_evaluation.md](docs/accuracy_evaluation.md) — scientific methodology & ULP evaluation
- [docs/io-format.md](docs/io-format.md) — `inputs.json` in, `BENCH ` CSV out
- [docs/running-benchmarks.md](docs/running-benchmarks.md) — build/flash/run per platform
- [docs/platforms.md](docs/platforms.md) — chips, target triples, timing sources, status
- [docs/hil-setup.md](docs/hil-setup.md) — probe + GitLab HIL CI provisioning

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
4. A case in `benchmarks/inputs.json`
5. A `BenchmarkTask` subclass in `tools/inputs_generator/tasks.py`

You never touch the platform `main.rs` files — the macro weaves tasks in. Full walkthrough:
[docs/adding_a_benchmark.md](docs/adding_a_benchmark.md).

## Crate map (`benchmarks/` Cargo workspace)

- `mrs-benchmark-core` — `no_std`. Traits, task/input registry, per-library suites. The heart.
- `mrs-benchmark-macros` — proc-macros reading `inputs.json` at build time.
- `mrs-benchmark-host` — native runner, times in `ns`.
- `mrs-benchmark-stm32` / `mrs-benchmark-rp2040` / `mrs-benchmark-rp2350` — `no_std` firmware, time in `cycles`, RTT out. (rp2350 = Pico 2, Cortex-M33: DWT timing like stm32, but boots via an `IMAGE_DEF` block, not boot2.)
- `mrs-benchmark-esp32s3` — `no_std` firmware, Xtensa (not ARM), CCOUNT register timing, RTT out via native USB JTAG. Needs the `espup`-installed `esp` Rust toolchain, not plain `rustup target add`. **Not a `benchmarks/` workspace member**: `esp-hal`'s `riscv-rt` version conflicts with `rp235x-hal`'s, so it has its own standalone `Cargo.lock` (still built from its own crate dir like the others). Skips the `crazyflie-fw` library (that suite's C build only cross-links for ARM).
- `mrs-benchmark-crazyflie-sys` — the C side. `build.rs` uses `cc` + `bindgen` to compile the
  Crazyflie firmware math (`vendor/crazyflie-firmware` submodule, plus local `c_src/` wrappers)
  and a subset of CMSIS-DSP (`vendor/CMSIS-DSP`, its own top-level submodule — CMSIS-Core
  headers still come from crazyflie-firmware's nested CMSIS_5 checkout). Feeds both the
  `crazyflie-fw` and `cmsis-dsp` suites. ARM-only.
- `mrs-benchmark-collect` — greps `BENCH ` rows from logs → one merged `results.csv`.

## Commands

Prefer `cargo-make` (`cargo install cargo-make`), run from repo root:

```bash
cargo make inputs                           # generate inputs.json via tasks.py
cargo make generate-ref                     # compute f64 ground truth reference_results.json
cargo make update                           # update inputs.json and reference_results.json together
cargo make bench-host                       # native, ns
cargo make bench-stm32                      # STM32F405 over probe-rs, cycles
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

- **Code comments: rare and one line.** Only comment what the code can't say itself — a
  non-obvious *why*, a hidden invariant, a workaround. One line each, plain ASCII, no
  multi-line blocks or banners. Background, measurements, and rationale go in the commit
  message or `docs/`, not above the code.
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
- **`panic = "abort"`** in every profile (workspace `Cargo.toml`). Three build profiles ship:
  `release` (opt-level 3), `lto` (adds fat LTO + `codegen-units=1`), `size` (`opt-level="z"` on
  top of LTO). CI builds and runs all three on every board; `profile` is a CSV column.
- **The valid profile for a C-vs-Rust number depends on the task's ABI class**, and there is no
  single right one: free-ABI at `lto`, `mat33` by-value and pointer-ABI at `xlto`. `release` bills
  C for call overhead Rust doesn't pay (CrossProduct 2.70x); `lto` gives Rust fat LTO and the C
  object nothing (MatMul9x9 2.21x, but 0.98x at `xlto`); `size` inverts linalg to C outright
  (0.63x). At matched ABI and profile the two languages land within ~25%, both directions. Table
  in [docs/task-categories.md](docs/task-categories.md#which-profile-makes-a-c-vs-rust-number-valid).
  Cite the profile with every cross-language claim.
- **A task touching a transcendental measures the linked math provider, not the language.** The
  Rust `libm` crate loses to newlib on `sqrt` (1.8x) and `sincos` (10x) and wins on `atan2`, `exp`
  and `ln`. See
  [docs/task-categories.md](docs/task-categories.md#which-transcendental-provider-you-link-decides-the-cost).
- **Never quote `crazyflie-fw` composite ULP as accuracy.** The f64 reference follows the Rust
  operation order, so `EkfStep`'s 789,913 ULP is divergence from a different filter, not error.
  See [docs/accuracy_evaluation.md](docs/accuracy_evaluation.md#the-composite-reference-is-not-neutral).
- **The `xlto` profile is the one where C actually gets inlined, but it's not a blanket upgrade
  over `lto`.** clang + `-Clinker-plugin-lto`, one ThinLTO over C and Rust together (`cargo make
  bench-stm32-xlto`, `bench-rp2350-xlto`). CI data (stm32, full input sweep) closes `MatMul3x3`
  (2.76x → 1.16x) and `QuatToRotMatrix` (1.76x → 1.22x), and helps CMSIS-DSP's own `MatMul9x9`/
  `DotProduct64D` (~1.8x) and `EkfStep` (2.06x). But of 60 measured task x library pairs, 14
  regress by more than 3% (some over 20%), on both the C and pure-Rust side. Build it for code
  shaped like the wins above; don't swap it in as a default replacement for `lto`. Only for the two
  ARM hard-float targets. See [docs/task-categories.md](docs/task-categories.md) for the full
  breakdown and which row to quote at which profile.
- **On RP2040, the C suites' `sqrtf`/`sinf`/`cosf`/`atan2f`/`expf`/`logf` calls aren't real
  newlib.** `rustc` links `libcompiler_builtins.rlib` before a crate's requested `-lm`, and both
  are scanned lazily -- so compiler_builtins's own software fallback resolves those symbols
  before the linker ever reaches the real arm-none-eabi `libm.a`, even with a correct search
  path. Fixed on `stm32`/`rp2350-arm` (`mrs-benchmark-crazyflie-sys/build.rs` whole-archives the
  real `libm.a`). Not fixed on RP2040: it has no FPU, so fat LTO's identical-code-folding makes
  compiler_builtins's copy load-bearing for `mrs-benchmark-core`'s own Rust "libm" suite too,
  and whole-archiving collides with it instead of overriding it. See
  [docs/platforms.md](docs/platforms.md#rp2040-pico-1) for the full root cause.
- **A suite that can't build on every platform is Cargo-feature-gated, not assumed universal.**
  Both C suites (`crazyflie-fw` and `cmsis-dsp`) sit behind `mrs-benchmark-core`'s `crazyflie`
  feature (default on; the name predates the CMSIS-DSP suite and now under-describes it). A
  platform that can't build them disables `default-features` on its `mrs-benchmark-core`
  dependency. The macro also needs to know: `suite_feature_gate()` in
  `mrs-benchmark-macros/src/lib.rs` wraps those libraries' generated call sites in a matching
  `#[cfg(feature = ...)]`, since inputs.json is shared across all platforms and the generated
  runner otherwise references every library listed anywhere in it unconditionally.
- **`library_versions.json` is generated, not hand-maintained** -- `mrs-benchmark-collect`
  writes it next to `results.csv` (`library_versions()` in
  `mrs-benchmark-collect/src/lib.rs`), and the report legend falls back to a library's bare
  name if its key is missing. Rust crate versions come from `benchmarks/Cargo.lock`;
  `crazyflie-fw` and `cmsis-dsp` come from their pinned git submodule commits/tags instead —
  both are top-level submodules (`benchmarks/vendor/crazyflie-firmware`,
  `benchmarks/vendor/CMSIS-DSP`). `crazyflie-fw` tracks `master` with no tags, so it resolves
  via `git ls-tree` (works even without a checkout); `cmsis-dsp` is pinned to a real release tag, so
  it resolves via `git describe --tags` in the checked-out submodule instead, to report the
  human-readable tag rather than a bare SHA.

## State (2026-08, moves fast)

Host + RP2040 + RP2350 + STM32 + ESP32-S3 run and benchmark on hardware in GitLab CI
(RP2040/RP2350/ESP32-S3 on HIL), across all three build profiles.

**C is in.** Both C suites run on the ARM targets: `crazyflie-fw` (Bitcraze firmware math) and
`cmsis-dsp` (ARM's DSP library). ESP32-S3 skips both — that build only cross-links for ARM. So
milestone 1's C-vs-Rust comparison has data for 20 tasks × 6 libraries × 5 platforms × 3
profiles in `results.csv`, with ULP accuracy alongside it in `accuracy_results.csv`.

QP solvers and scheduling (FreeRTOS/Embassy) remain later milestones, not current work. Check
[docs/platforms.md](docs/platforms.md) for live status before assuming a target works.
