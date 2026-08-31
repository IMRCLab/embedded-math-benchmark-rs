# Task categories

What each task measures, and therefore which comparisons it supports. Check here before
quoting a number as a C-vs-Rust result.

`ABI` is what the C wrapper's signature costs at the FFI boundary. See
[FFI cost is about struct shape](#ffi-cost-is-about-struct-shape).

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

**arithmetic**: C-vs-Rust codegen, but only on the free-ABI rows. Quote a `mat33` row as a
language result and you are mostly quoting AAPCS struct copies. At `xlto` this inverts:
`MatMul3x3` and `QuatToRotMatrix` become clean and the free-ABI rows become the contaminated ones.

**libm-bound**: nothing about language codegen. These rows measure the *transcendental provider*, and C wins both outright. The gap is inherited whole from the transcendental rows, so quote those instead unless you specifically want the composite. See [Which transcendental provider you link decides the cost](#which-transcendental-provider-you-link-decides-the-cost).

**transcendental**: libm implementations against each other, newlib vs the Rust `libm` crate vs micromath. The `crazyflie-fw` rows here hold no Crazyflie code; they are the ARM toolchain's newlib, see [c-suites.md](c-suites.md#naming). On rp2040 they are not even that, see [platforms.md](platforms.md#rp2040-pico-1).

**linalg**: no by-value marshalling (CMSIS-DSP takes `arm_matrix_instance_f32*`, and its routines are non-inline library calls in real use too), but *profile-dominated*: at `lto` Rust gets fat LTO and the C object gets nothing. `MatMul9x9` reads 2.21x Rust at `lto` and 0.98x at `xlto`. Quote it at `xlto`.

**composite**: what a realistic control-loop step costs per platform. Not a language comparison, because the C and Rust paths are different algorithms rather than two translations of one. The ULP data shows this directly, but the f64 reference follows the Rust operation order, so `crazyflie-fw`'s composite ULP is divergence rather than error. See [accuracy_evaluation.md](accuracy_evaluation.md#the-composite-reference-is-not-neutral). `LeeController` reports `[thrust (N), torque_x, torque_y, torque_z (N*m)]`, what the controller itself computes; motor mixing is a separate algorithm and out of scope.

## Which profile makes a C-vs-Rust number valid

Each ABI class is only a language comparison at the profile where both sides get equivalent compiler treatment. Read the row there; say which one you used.

| ABI class | Tasks | Read at | C/best-Rust there | Why not the others |
| --------- | ----- | ------- | ----------------- | ------------------ |
| free (registers) | `CrossProduct` `QuatMul` `UnitQuatMul` `RotateVector` | `lto` | 1.07-1.27x | `release` leaves Rust-side scaffolding un-inlined (2.70x on `CrossProduct`); `xlto` slows cfw's own call site 26% |
| `mat33` by-value | `MatMul3x3` `QuatToRotMatrix` | `xlto` | 1.16-1.22x | `lto` bills C for AAPCS struct copies (2.76x) |
| pointer | `MatMul9x9` `MatInverse9x9` `DotProduct64D` | `xlto` | 0.94-1.47x | `lto` gives Rust fat LTO and the C object nothing (2.21x) |

Measured on stm32, `results.csv` of 2026-08-26. **At matched ABI and profile, C and Rust land within ~25% of each other, in both directions.** Any larger number is a profile or ABI artifact rather than a language result.

`size` inverts the linalg rows outright: `MatMul9x9` 0.63x, `MatInverse9x9` 0.64x. nalgebra's compile-time unrolling is what `opt-level="z"` throws away, so C wins flash-constrained builds.

## Which transcendental provider you link decides the cost

`glam` and `nalgebra` are built with `features = ["libm"]` (there is no `f32::sqrt` in `core`), so every transcendental in a pure-Rust suite routes through the `libm` crate. That crate is not uniformly slower than newlib. It loses badly on `Sqrt` and `SinCos`, and wins on `Atan2`, `Exp` and `Ln`. Which provider you link decides each function; the language does not.

ns per call, stm32 `lto`:

| Task | newlib | `libm` crate | micromath |
| ---- | ------ | ------------ | --------- |
| `Sqrt` | 233 | 423 (1.82x) | 209 |
| `SinCos` | 2154 | 21802 (10.12x) | 373 |
| `Atan2` | 1456 | 1091 (0.75x) | 388 |
| `Exp` | 1106 | 598 (0.54x) | 696 |
| `Ln` | 1007 | 566 (0.56x) | 478 |

The two losses have different causes, both visible in the disassembly. For `Sqrt`, newlib emits `vsqrt.f32` (14 of the 39 cycles its 233 ns spans; the rest is `sqrtf`'s NaN and errno guard plus the call), while the `libm` crate's architecture dispatch has no 32-bit ARM backend and falls back to a 128-entry table lookup with integer Goldschmidt iterations. For trigonometry no core here has a hardware instruction, so newlib evaluates single-precision polynomials on the FPU via `vfma.f32`, and the `libm` crate evaluates them in `f64`, which forces `rustc` to emit `__aeabi_dmul` software emulation on a single-precision FPU. That is the whole 10x.

rp2350-arm has the same shape (`Sqrt` 2.23x, `SinCos` 10.43x, and the same three wins). esp32s3 links no C library here, so the comparison there is `libm` against micromath: 2.72x on `Sqrt`. rp2040 is a different picture rather than a fourth data point, because its C side resolves to `compiler_builtins` rather than newlib (see [platforms.md](platforms.md)); `libm` loses 4.48x on `Exp` and 2.94x on `Ln` there, and roughly ties on `SinCos` and `Atan2`.

Wherever the ratio is large it carries straight into any task that touches that function:

- `Vec3Normalize`: glam - cfw = 163 ns; `Sqrt`: libm - cfw = 190 ns. Same gap.
- `QuatSlerp` (`acos` + 3 `sin`): glam/cfw = 10.35x; `SinCos`: libm/cfw = 10.12x. Same ratio.

micromath is the fastest provider on every transcendental except `Exp`, at the accuracy cost in [accuracy_evaluation.md](accuracy_evaluation.md).

## FFI cost is about struct shape

What costs real cycles at the FFI boundary is AAPCS-VFP by-value struct passing, not the wrapper
call itself (see [c-suites.md](c-suites.md)). `struct vec` and `struct quat` are
homogeneous float aggregates, so they ride in `s0`-`s3` and cost nothing. `struct mat33` is 36
bytes: arguments get copied onto the stack, results come back through a hidden `sret` pointer.
Rust keeps the same values in registers, so every `mat33` at the boundary is memory traffic
Rust never pays.

Measured on stm32 / `lto`, cycles per call, against the memory ops in the C wrapper body:

| Task              | mem ops in wrapper | `crazyflie-fw` | `glam` | ratio |
| ----------------- | ------------------ | -------------- | ------ | ----- |
| `UnitQuatMul`     | 0 of 24            | 75.8           | 71.4   | 1.06x |
| `QuatMul`         | 0 of 24            | 76.0           | 68.5   | 1.11x |
| `CrossProduct`    | 0 of 13            | 41.8           | 35.3   | 1.18x |
| `MatVecMul3x3`    | 10 of 30           | 97.3           | 56.3   | 1.73x |
| `QuatToRotMatrix` | 9 of 36            | 106.2          | 60.4   | 1.76x |
| `MatMul3x3`       | 42 of 93           | 349.6          | 126.5  | 2.76x |

The free-ABI rows are trustworthy C-vs-Rust codegen. The `mat33` rows are not: in `MatMul3x3`
nearly half the C path is marshalling, and the timed loop's whole pre-call body is `ldrd`/`strd`
copying two matrices onto the stack.

## Cross-language LTO in one paragraph

`xlto` closes the `mat33` by-value rows (`MatMul3x3` 2.76x to 1.16x) and helps CMSIS-DSP's larger
routines, but of 60 measured stm32 task x library pairs 14 regress by more than 3%, on both the C
and the pure-Rust side. It is not a default replacement for `lto`. See
[build-profiles.md](build-profiles.md#xlto-is-not-a-strict-win).
