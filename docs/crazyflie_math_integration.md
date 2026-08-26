# Crazyflie C Math Reference Library Integration

This document details the integration of the Crazyflie C math library (`math3d.h`/`cmath3d` from the [bitcraze/crazyflie-firmware](https://github.com/bitcraze/crazyflie-firmware) repository) into the `mrs-microbenchmarks` Cargo workspace.

The library is registered under the identifier `crazyflie-fw` and serves as a direct reference for performance comparisons between embedded C and Rust math libraries.

## Math provenance

Verified (2026-08-26) that every `crazyflie-fw` task calls genuine code, not a hand-rolled
stand-in: the wrapped `math3d.h` functions (`vcross`, `mmul`, `qqmul`, `qslerp`,
`quat2rotmat`, `qnormalize`, ...) are textbook formulas, cited to their sources in comments;
`cf_ekf_step`/`cf_lee_controller` call the real `kalman_core.c`/`controller_lee.c`. Crazyflie
firmware does contain a classic Quake fast-inverse-sqrt (`sensfusion6.c`'s `invSqrt`, magic
constant `0x5f3759df`) but that file is never compiled into this crate (not in `build.rs`'s
file list) -- every sqrt path here (`vnormalize`, `arm_sqrt` in `kalman_core.c`, CMSIS's own
`arm_sqrt_f32` on GCC/clang) resolves to plain `sqrtf`.

`Sqrt`/`SinCos`/`Atan2`/`Exp`/`Ln` are the exception: they call `sinf`/`cosf`/etc. directly, so
`crazyflie-fw` there is really just the ARM toolchain's linked `newlib`, no firmware code runs
at all (see [task-categories.md](task-categories.md)'s transcendental row). The paper and
`viz/paper_figures.py` label that column `newlib` instead, to keep it visually distinct from
tasks (like `Vec3Normalize`) where `crazyflie-fw` genuinely does run firmware code.

---

## Required Installs & Dependencies

To compile and execute the benchmarks for microcontrollers (such as STM32, RP2040, or RP2350) locally, you must install the target compilers, standard libraries, and Rust target triples.

### 1. Cross-Compiler & C Standard Library

#### Arch Linux
Install the GNU ARM toolchain and the associated C standard library (`newlib`):
```bash
# Install ARM GCC cross-compiler
paru -S arm-none-eabi-gcc

# Install target-specific standard library headers and archives (provides math.h, libm.a)
paru -S arm-none-eabi-newlib
```

#### Ubuntu / Debian
```bash
sudo apt-get update
sudo apt-get install gcc-arm-none-eabi libnewlib-arm-none-eabi
```

### 2. Rustup Target Architectures
Ensure your local Rustup toolchain has the target standard libraries installed for the microcontrollers:
```bash
rustup target add thumbv7em-none-eabihf thumbv6m-none-eabi thumbv8m.main-none-eabihf
```

---

## FFI Integration Details (`mrs-benchmark-crazyflie-sys`)

Because Crazyflie's `math3d.h` is a header-only library where functions are declared as `static inline`, they do not produce linker symbols in compiled object files. 

To bridge this to Rust, we created a system FFI crate:
1. **C Wrapper File (`cf_math_wrapper.c`)**: Exposes the inline math functions under standard non-inline symbols (`cf_mmul`, `cf_qrot` (which wraps `qvrot`), `cf_qqmul`, `cf_qslerp`).
2. **Header Entry (`crazyflie_fw.h`)**: Groups headers for the `bindgen` scanner to generate Rust FFI bindings.

### `LeeController` reference (`cf_lee_controller.c`)

Wraps `modules/src/controller/controller_lee.c` directly (a real `.c` file, compiled as its
own source in `build.rs`, not a header-only inline). Builds the firmware's structs from flat
args, forces the position-control branch, and returns `{ thrust, torque }`: the firmware's
motor mixing is a separate, non-sqrt algorithm, out of scope here. Also zeroes `self.KI`
after init, since a single call from a fresh reset still picks up one `dt`-step of integral
windup the other 3 implementations don't model.

---

## Cross-Compilation Workarounds

Integrating C FFI into a cross-compiled `#![no_std]` Rust binary requires several custom configurations in [mrs-benchmark-crazyflie-sys/build.rs](file:///home/ph/git/mrs-microbenchmarks/benchmarks/mrs-benchmark-crazyflie-sys/build.rs):

### 1. Dynamic GCC `libm.a` Resolution
**Problem**: The standard math functions (like `sinf`, `cosf`) are linked via `-lm`. However, Rust's linker (`rust-lld`) does not automatically search the system's cross-compiler paths, causing a `unable to find library -lm` link failure.  
**Solution**: The build script queries the system's `arm-none-eabi-gcc` using the target FPU architecture flags and `-print-file-name=libm.a`:
```bash
arm-none-eabi-gcc -mthumb -march=armv7e-m -mfloat-abi=hard -mfpu=fpv4-sp-d16 -print-file-name=libm.a
```
It extracts the parent directory of the returned path and dynamically registers it with Cargo using `cargo:rustc-link-search=native=<path>`. This ensures the linker resolves the optimized, target-specific `libm.a` correctly.

### 2. Bindgen Target Triple Matching
**Problem**: When `bindgen` invokes Clang to parse the C headers, it defaults to the host target (e.g. `x86_64`), leading to incorrect pointer sizes and structural layouts in the generated Rust FFI file.  
**Solution**: We explicitly pass the active target triple to Clang using the `.clang_arg(format!("--target={}", target))` builder flag.

### 3. Opaque/System Struct Size Assertions
**Problem**: Clang's parser generated size assertions for internal libc structs (such as `_reent`) that caused compile-time panics due to size mismatches between host and target environments.  
**Solution**: We disabled layout test generation in bindgen using `.layout_tests(false)`. Since we only pass custom 3D math types (`vec`, `mat33`, `quat`) and never instantiate or pass internal libc structures, this is completely safe.

### 4. Warning Suppression
Suppress warnings from the third-party Crazyflie headers (e.g., `-Wstrict-aliasing` and `-Wabsolute-value` warnings in `math3d.h`) during compiler optimization passes by compiling the wrapper with:
* `.flag_if_supported("-Wno-absolute-value")`
* `.flag_if_supported("-Wno-strict-aliasing")`

### 5. System Header Discovery for `bindgen`
**Problem**: `bindgen`'s Clang doesn't know where the cross-compiler's headers (newlib's
`math.h`) live — it's a packaging detail that differs per distro (Arch sysroots under
`/usr/arm-none-eabi/include`; Debian under `/usr/lib/arm-none-eabi/include`), and no single
gcc flag reports it everywhere (`-print-sysroot` works on Arch, returns empty on Debian).
**Solution**: Not elegant, but portable — make gcc resolve the actual wrapper header via
`gcc -M` (standard dependency-list output) and read the header directories off its answer,
instead of asking gcc to describe its general config. Verified on both distros, including
the Debian-based CI image (see below).

### 6. Kconfig-Generated `autoconf.h`
**Problem**: `controller_lee.c` needs `platform_defaults.h`, which includes `autoconf.h`,
normally Kconfig-generated at firmware build time. Not present in the vendored submodule.
**Solution**: `platform_defaults.h` already falls back to its own `#ifndef CF_MASS` / ...
defaults when no `CONFIG_PLATFORM_*` is set, so an empty stub (`c_src/stub_autoconf/autoconf.h`)
is enough; the wrapper overrides `self.mass` right after `controllerLeeInit()` anyway.

---

## CI Integration

Building this crate on CI needs `arm-none-eabi-gcc`/`libnewlib-arm-none-eabi`, provided
by a custom image — see [hil-setup.md](hil-setup.md#ci-image). `libnewlib-dev` (the actual
header package) comes along for free as `libnewlib-arm-none-eabi`'s hard dependency; no
separate Dockerfile entry needed.

### Submodule fetch
`GIT_SUBMODULE_STRATEGY: normal` + `GIT_SUBMODULE_DEPTH: 1` (in `.gitlab-ci.yml`) fetch only
the top-level submodules declared in the repo's own `.gitmodules`: `crazyflie-firmware` and
`CMSIS-DSP`. It does not recurse into crazyflie-firmware's own nested submodules (`vendor/CMSIS`,
`FreeRTOS`, `cmock`, `unity`, `libdw1000`) — none of which this suite touches; CMSIS-DSP is
vendored as our own top-level submodule instead of reusing crazyflie-firmware's nested (and
long-frozen) CMSIS_5 copy, so `normal` already covers it without switching to `recursive`.

### Testing against the CI image locally
`.gitlab/ci/Dockerfile.ci` builds and runs directly, no registry needed:
```bash
docker build -t mrs-ci-test -f .gitlab/ci/Dockerfile.ci .gitlab/ci
docker run --rm --user "$(id -u):$(id -g)" -v "$(pwd)":/work -w /work/benchmarks \
  -e CARGO_TARGET_DIR=/tmp/ci-target -e CARGO_HOME=/tmp/cargo-home \
  mrs-ci-test cargo build -p mrs-benchmark-crazyflie-sys --target thumbv7em-none-eabihf
```
Keep `--user` + the redirected `CARGO_TARGET_DIR`/`CARGO_HOME` — without them the container
runs as root and litters the host's bind-mounted `target/` with root-owned files.

