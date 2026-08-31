# Adding a new platform

Checklist for bringing up a new hardware target (or new architecture on existing silicon).

## Checklist

1. **New crate** `mrs-benchmark-<platform>`, own dir, dir name == crate/binary name.
   - `Cargo.toml`: depends on `mrs-benchmark-core`; `default-features = false` on that dep
     if a suite can't cross-compile here (e.g. `crazyflie-fw` needs an ARM libm cross-link).
   - `.cargo/config.toml`: pins the target triple and `rustflags` (linker script,
     `--nmagic`, ISA/CPU flags). Crate-local, so it doesn't leak into the workspace build.
   - Linker script plus a `build.rs` copying it into `OUT_DIR`, if needed beyond what the
     runtime crate ships.
   - `src/main.rs`: implement `BenchmarkPlatform` (`id()`, `setup()`, `now()`/`elapsed()`,
     `unit()`, `log_result()`); pass `mrs_benchmark_core::PROFILE` as the profile field in
     `log_result()`. Pick a timing source, see [platforms.md](platforms.md). Add
     boot metadata the chip's boot ROM needs, if any. Set the CPU clock explicitly if the
     HAL default isn't nominal: a raw cycle count is meaningless without the Hz. Use
     `rtt_target`'s `ChannelMode::BlockIfFull` with an explicit buffer size; the default can
     silently drop output under load.
   - `rust-toolchain.toml` only if the target needs a non-stock toolchain.
   - Never touch this crate to add a *benchmark*, only to add a *platform*.

2. **Workspace membership** (`benchmarks/Cargo.toml`): add to `members`, unless the
   toolchain's deps conflict with another crate already there. Then keep a standalone
   `Cargo.lock`, still built by `cd`-ing into the crate dir.

3. **Suite gating**, only if a platform can't build an existing suite: no new plumbing.
   `mrs-benchmark-macros`' `suite_feature_gate()` and `suites.rs`'s `#[cfg(feature = ...)]`
   already handle it. Flip `default-features = false` on the new crate's
   `mrs-benchmark-core` dependency.

4. **CI** (`.gitlab/ci/`):
   - `Dockerfile.ci`: `rustup target add`, or a separate toolchain if not upstream Rust.
   - `build.yml`: `build-<platform>` extending `.cargo-build` (`BIN`, `TARGET`). Automatically
     builds `release`, `lto`, and `size` profiles via parallel matrix.
   - `check.yml`: `clippy-<platform>`, mirroring the build job.
   - `run.yml`: `run-<platform>` extending `.firmware-flash` (`CHIP`, `BIN`, `TARGET_NAME`,
     `PROBE`), once flashing works over `probe-rs`. Runs all 3 profiles sequentially.
   - `collect.yml`: add `- job: run-<platform>` / `optional: true` to `collect-results`.
   - A `HIL_PROBE_<PLATFORM>` project variable holding the probe's selector from
     `probe-rs list`, referenced as `PROBE` by the run job. See
     [hil-setup.md](hil-setup.md#probe-selection).

5. **Local dev** (`Makefile.toml`): `build-<alias>` and `bench-<alias>` tasks mirroring the
   CI jobs.

6. **Reporting** (`viz/config.py`, outside the Rust tree, easy to forget):
   - `PLATFORM_CLOCK_HZ["<platform-id>"] = <hz>`, matching `main.rs` exactly. Only for
     `cycles`-unit platforms; `host` reports `ns` natively and needs nothing here.
   - `PLATFORM_ORDER`: add the platform id (before `"host"`, which always sorts last)
     for stable report-page ordering.

7. **Docs**: `platforms.md` (table row + footnote), `hil-setup.md` (if provisioning is
   unusual), `CLAUDE.md` (crate map, `Commands`, `State`).

## Not always needed

- A C cross-compiler (only for `crazyflie-fw`).
- Workspace membership (see step 2).
