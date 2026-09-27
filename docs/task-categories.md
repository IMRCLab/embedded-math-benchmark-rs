# Task categories

What each task measures, and therefore which comparisons it supports. Check here before quoting a number as a C-vs-Rust result. `ABI` is what the C wrapper's signature costs at the FFI boundary, see [FFI cost is about struct shape](#ffi-cost-is-about-struct-shape).

| Task                                        | Category       | ABI                     | Why                                                          |
| ------------------------------------------- | -------------- | ----------------------- | ------------------------------------------------------------ |
| `CrossProduct`                              | arithmetic     | free                    | `vcross`, pure mul/sub                                       |
| `QuatMul`                                   | arithmetic     | free                    | `qqmul`                                                      |
| `UnitQuatMul`                               | arithmetic     | free                    | `qqmul`                                                      |
| `RotateVector`                              | arithmetic     | free                    | `qvrot`, built from `vdot`/`vcross`                          |
| `MatVecMul3x3`                              | arithmetic     | one `mat33` in          | `mvmul`                                                      |
| `QuatToRotMatrix`                           | arithmetic     | one `mat33` out         | `quat2rotmat`                                                |
| `MatMul3x3`                                 | arithmetic     | two `mat33` in, one out | `mmul`, triple loop                                          |
| `MatInverse3x3`                             | arithmetic     | n/a                     | Rust-only; source of the `ERROR` rows (near-singular inputs) |
| `Vec3Normalize`                             | libm-bound     | free                    | `vnormalize` -> `vmag` -> `sqrtf`                            |
| `QuatSlerp`                                 | libm-bound     | free                    | `qslerp` -> `acosf` + three `sinf`                           |
| `Sqrt` `SinCos` `Atan2` `Exp` `Ln`          | transcendental | free                    | direct libm entry points                                     |
| `DotProduct64D` `MatMul9x9` `MatInverse9x9` | linalg         | pointers                | 9x9 / 64-element operands                                    |
| `EkfStep`                                   | composite      | pointers                | `kalman_core.c`: CMSIS `arm_mat_*` plus 18 `powf`            |
| `LeeController`                             | composite      | pointers                | `controller_lee.c`: many primitives plus `sinf`/`cosf`       |

## What each category supports

**arithmetic**: C-vs-Rust codegen, but only at the profile its ABI class allows (table below). Read at the wrong profile, a `mat33` row measures AAPCS struct copies and a free-ABI row measures `xlto`'s codegen.

**libm-bound**: the transcendental provider, not language codegen. The gap is inherited from the transcendental rows, so quote those unless you specifically want the composite.

**transcendental**: newlib, the Rust `libm` crate and micromath against each other. See [Which transcendental provider you link decides the cost](#which-transcendental-provider-you-link-decides-the-cost).

**linalg**: pointer ABI, so no marshalling (CMSIS-DSP takes `arm_matrix_instance_f32*`, and its routines are non-inline library calls in real use too), but profile-dominated: at `lto` Rust gets fat LTO and the C object gets nothing. Quote it at `xlto`.

**composite**: what a realistic control-loop step costs per platform. Not a language comparison: the C and Rust paths are different algorithms, and `crazyflie-fw`'s composite ULP is divergence rather than error, see [accuracy_evaluation.md](accuracy_evaluation.md#the-composite-reference-is-not-neutral). `LeeController` reports `[thrust (N), torque_x, torque_y, torque_z (N*m)]`, what the controller itself computes; motor mixing is out of scope.

## Which profile makes a C-vs-Rust number valid

Each ABI class is only a language comparison at the profile where both sides get equivalent compiler treatment. Read the row there and say which one you used.

| ABI class | Tasks | Read at | Why not the others |
| --------- | ----- | ------- | ------------------ |
| free (registers) | `CrossProduct` `QuatMul` `UnitQuatMul` `RotateVector` | `lto` | `release` leaves Rust-side scaffolding un-inlined; `xlto` slows cfw's own call site |
| `mat33` by-value | `MatMul3x3` `QuatToRotMatrix` | `xlto` | `lto` bills C for AAPCS struct copies |
| pointer | `MatMul9x9` `MatInverse9x9` `DotProduct64D` | `xlto` | `lto` gives Rust fat LTO and the C object nothing |

`MatVecMul3x3` has no valid profile: its vector return keeps the C behind a call even at `xlto`, see [build-profiles.md](build-profiles.md#cross-language-lto-xlto).

**At matched ABI and profile, C and Rust land close together, in both directions.** A large gap is a profile or ABI artifact rather than a language result.

`size` inverts the linalg rows: C wins both 9x9 tasks, because `opt-level="z"` throws away nalgebra's compile-time unrolling.

## FFI cost is about struct shape

What costs real cycles at the FFI boundary is AAPCS-VFP by-value struct passing, not the wrapper call itself (see [c-suites.md](c-suites.md)). `struct vec` and `struct quat` are homogeneous float aggregates, so they ride in `s0`-`s3` and cost nothing. `struct mat33` is 36 bytes: arguments get copied onto the stack, results come back through a hidden `sret` pointer. Rust keeps the same values in registers, so every `mat33` at the boundary is memory traffic Rust never pays.

A `mat33` row at `lto` therefore measures marshalling, not codegen. `fig-results-ffi.pdf` from `viz/paper_figures.py` plots the C/Rust ratio against each wrapper's load/store share.

## Which transcendental provider you link decides the cost

`glam` and `nalgebra` are built with `features = ["libm"]` (there is no `f32::sqrt` in `core`), so every transcendental in a pure-Rust suite routes through the `libm` crate. That crate loses to newlib on `Sqrt`, and badly on `SinCos` because it evaluates in `f64`, which a single-precision FPU emulates in software. It wins on `Atan2`, `Exp` and `Ln`, with bit-identical results. The provider decides each function; the language does not.

The `crazyflie-fw` rows here hold no Crazyflie code: they are the ARM toolchain's newlib, see [c-suites.md](c-suites.md#naming). On rp2040 they resolve to `compiler_builtins` instead, see [platforms.md](platforms.md#rp2040-pico-1). esp32s3 links no C library, so there the comparison is `libm` against micromath.
