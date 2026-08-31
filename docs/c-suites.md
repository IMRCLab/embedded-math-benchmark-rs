# C reference suites

Two C suites run on the ARM targets, both behind `mrs-benchmark-core`'s `crazyflie` feature
(default on; the name predates the CMSIS-DSP suite and now under-describes it). `crazyflie-fw`
wraps the Bitcraze firmware's `math3d.h`, `kalman_core.c` and `controller_lee.c` from the
`benchmarks/vendor/crazyflie-firmware` submodule. `cmsis-dsp` wraps a subset of ARM's DSP library
from `benchmarks/vendor/CMSIS-DSP`. Both are compiled by `mrs-benchmark-crazyflie-sys`'s
`build.rs` with `cc` and `bindgen`. esp32s3 skips both: that build only cross-links for ARM.

`math3d.h` is header-only with `static inline` functions, which produce no linker symbols, so
`c_src/cf_math_wrapper.c` re-exposes them under non-inline names (`cf_mmul`, `cf_qqmul`,
`cf_qslerp`, `cf_qrot`, ...). What survives into the timed region is one wrapper call. The call
itself is a rounding error; the by-value `mat33` marshalling around it is not, see
[task-categories.md](task-categories.md#ffi-cost-is-about-struct-shape).

## Math provenance

Every `crazyflie-fw` task calls genuine firmware code rather than a hand-rolled stand-in. The
wrapped `math3d.h` functions are textbook formulas cited to their sources in comments, and
`cf_ekf_step`/`cf_lee_controller` call the real `kalman_core.c`/`controller_lee.c`. The firmware's
Quake fast-inverse-sqrt (`sensfusion6.c`'s `invSqrt`) is never compiled into this crate, so every
sqrt path here resolves to plain `sqrtf`.

## Naming

`Sqrt`, `SinCos`, `Atan2`, `Exp` and `Ln` are the exception: they call `sinf`/`cosf`/etc.
directly, so no firmware code runs and the `crazyflie-fw` column there is really the ARM
toolchain's newlib. The workshop paper and `viz/paper_figures.py` label those rows `newlib`, and
the paper renames the suite itself `cmath3d`. The CSV keeps `crazyflie-fw` for both.

## `LeeController` wrapper

`c_src/cf_lee_controller.c` wraps `modules/src/controller/controller_lee.c` as a real compiled
source, not a header-only inline. It builds the firmware's structs from flat args, forces the
position-control branch, and returns `{ thrust, torque }`; the firmware's motor mixing is a
separate algorithm and out of scope. It zeroes `self.KI` right after `controllerLeeInit()`, since
a single call from a fresh reset would otherwise pick up one `dt`-step of integral windup that the
three Rust implementations do not model.

`platform_defaults.h` wants a Kconfig-generated `autoconf.h` that the vendored submodule does not
carry. Its own `#ifndef` fallbacks suffice, so `c_src/stub_autoconf/autoconf.h` is empty.

## Submodules in CI

`GIT_SUBMODULE_STRATEGY: normal` plus `GIT_SUBMODULE_DEPTH: 1` fetch only the two top-level
submodules this repo declares. It does not recurse into crazyflie-firmware's own nested submodules,
none of which these suites touch. CMSIS-DSP is vendored as our own top-level submodule rather than
reusing crazyflie-firmware's nested and long-frozen CMSIS_5 copy, so `normal` already covers it.
