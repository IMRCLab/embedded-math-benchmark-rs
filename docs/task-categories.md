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

## Cross-language LTO (`xlto` profile)

`cargo make bench-stm-xlto`, `cargo make bench-pico2-xlto`. clang compiles the C to bitcode, rustc
emits bitcode under `-Clinker-plugin-lto`, and rust-lld runs one ThinLTO over both. The profile is
based on `release`, not `lto`: fat LTO runs inside rustc and the C bitcode never joins that link, so
cross-LTO there silently does nothing.

Three things must line up or the C stays behind a call:

- **`-C target-cpu` must match clang's `-mcpu`**, or `ARMTTIImpl::areInlineCompatible` rejects the
  callee on target features.
- **`-import-instr-limit`** defaults to 100, too low for the bigger wrappers; we pass 500. Below
  that the inliner reports `NoDefinition` and never sees a body.
- **The LLVM function types must match.** rustc lowers a by-value HFA to `[3 x float]` /
  `[4 x float]`; clang keeps `%struct.vec` / `%struct.quat`. `InlineFunction` rejects the mismatch
  before cost analysis, so no remark is emitted and `alwaysinline` cannot override it. `ptr`,
  `float` and `[9 x i32]` params agree, and `sret` is an attribute rather than part of the type.

Nine of the eleven wrappers pass or return a `vec`/`quat` by value and hit the third rule.
`cf_quat2rotmat` was the only free fix: its return was already `sret`, so scalarizing the quat
argument into four `float`s (same registers, same ABI) was enough. The others mismatch on the
*return*, which only an out-pointer fixes, and that would cost the other profiles real memory
traffic. `cf_mmul` never mismatched at all; it was blocked only by the import limit.

Measured on stm32, median cycles per call, one session:

| Task              | `lto` cfw | `xlto` cfw | `lto` glam | `xlto` glam | ratio           |
| ----------------- | --------- | ---------- | ---------- | ----------- | --------------- |
| `MatMul3x3`       | 352.5     | 146.2      | 127.4      | 126.0       | 2.77x -> 1.16x  |
| `QuatToRotMatrix` | 108.5     | 50.4       | 60.7       | 50.7        | 1.79x -> 1.00x  |
| `MatVecMul3x3`    | 96.6      | 91.3       | 57.2       | 44.0        | 1.69x -> 2.08x  |
| `CrossProduct`    | 44.8      | 42.6       | 39.0       | 31.5        | 1.15x -> 1.36x  |

`xlto` closes the two rows it can inline and widens the rest, because Rust gets inlined under the
same regime while the C stays behind a call. Read each row at the profile where its wrapper can
actually be inlined, and say which one you used.

Swapping gcc for clang is worth 1.05x (`CrossProduct`) to 1.33x (`MatMul3x3`) on its own, measured
at `lto` with no cross-LTO. gcc was never a deliberate choice; it is the `cc` crate's default for
`thumb*-none-eabi*`.
