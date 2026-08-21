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
language result and you are mostly quoting AAPCS struct copies.

**libm-bound**: nothing about language codegen. The result reflects which libm is linked, so
read it next to the transcendental rows. Note C wins both of these outright.

**transcendental**: libm implementations against each other, newlib vs the Rust `libm` crate vs
micromath. The `crazyflie-fw` rows here hold no Crazyflie code; they are the ARM toolchain's
newlib. In results predating `84d6028` they are not even that, because `compiler_builtins`'
weak symbols shadowed the real `libm.a`. See [platforms.md](platforms.md).

**linalg**: the cleanest C-vs-Rust comparison in the suite. CMSIS-DSP takes
`arm_matrix_instance_f32*`, so there is no by-value marshalling, and its routines are non-inline
library calls in real use too.

**composite**: what a realistic control-loop step costs per platform. Not a language
comparison, because the C and Rust paths are different algorithms rather than two translations
of one. The ULP data shows this directly.

## FFI cost is about struct shape

`math3d.h`'s `static inline` functions _are_ inlined, into `cf_math_wrapper.c`, at its own
compile step. What survives into the timed region is one wrapper call, and the call itself
(`bl` + `bx lr`) is a rounding error.

What costs real cycles is AAPCS-VFP by-value struct passing. `struct vec` and `struct quat` are
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
| `QuatToRotMatrix` | 9 of 36            | 112.3          | 60.4   | 1.86x |
| `MatMul3x3`       | 42 of 93           | 349.6          | 126.5  | 2.76x |

The free-ABI rows are trustworthy C-vs-Rust codegen. The `mat33` rows are not: in `MatMul3x3`
nearly half the C path is marshalling, and the timed loop's whole pre-call body is `ldrd`/`strd`
copying two matrices onto the stack.

Cross-language LTO fixes part of this. Point `cc` at clang with `-flto=thin`, build Rust with
`-Clinker-plugin-lto`, and rust-lld runs one ThinLTO over both. Two separate things then block
inlining:

- **ThinLTO import limit.** `-import-instr-limit` defaults to 100, so the bigger wrappers are never
  imported into the calling module and the inliner reports `NoDefinition`. Passing
  `--mllvm=-import-instr-limit=10000` to the linker imports them and `cf_mmul` inlines away.
- **LLVM function-type mismatch.** rustc lowers a by-value `struct vec` to `[3 x float]`; clang keeps
  `%struct.vec`. Both are AAPCS-identical and link correctly, but `InlineFunction` rejects a call
  whose type differs from the callee's. That happens before cost analysis, so no remark is emitted
  and `alwaysinline` cannot override it. Wrappers taking only `ptr`, `float`, or `[9 x i32]` are
  unaffected, which is why `cf_mmul` inlines and `cf_vcross` does not (`sret` is an attribute, not
  part of the type).

Scalarizing the float aggregates in the wrapper signature (`float ax, float ay, ...` plus an
out-pointer) makes the types agree; the callee then inlines to bare `vmul`/`vsub`. Verified in a
minimal repro, not adopted here.

With the import limit raised, the `MatMul3x3` timed loop is 86 instructions for `crazyflie-fw`
against 77 for `glam`, both call-free, where the shipped numbers show a 2.76x cycle gap. Cycle
confirmation on hardware is still pending, and the numbers in `results.csv` are all built the
current way (gcc, no cross-language LTO).
