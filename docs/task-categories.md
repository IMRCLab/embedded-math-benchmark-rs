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

**transcendental**: libm implementations against each other, newlib vs the Rust `libm` crate vs micromath. The `crazyflie-fw` rows here hold no Crazyflie code; they are the ARM toolchain's newlib. In results predating `84d6028` they are not even that, because `compiler_builtins`' weak symbols shadowed the real `libm.a`. See [platforms.md](platforms.md).

**linalg**: no by-value marshalling (CMSIS-DSP takes `arm_matrix_instance_f32*`, and its routines are non-inline library calls in real use too), but *profile-dominated*: at `lto` Rust gets fat LTO and the C object gets nothing. `MatMul9x9` reads 2.21x Rust at `lto` and 0.98x at `xlto`. Quote it at `xlto`.

**composite**: what a realistic control-loop step costs per platform. Not a language comparison, because the C and Rust paths are different algorithms rather than two translations of one. The ULP data shows this directly, but the f64 reference follows the Rust operation order, so `crazyflie-fw`'s composite ULP is divergence rather than error. See [accuracy_evaluation.md](accuracy_evaluation.md#the-composite-reference-is-not-neutral).

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
| `QuatToRotMatrix` | 9 of 36            | 106.2          | 60.4   | 1.76x |
| `MatMul3x3`       | 42 of 93           | 349.6          | 126.5  | 2.76x |

The free-ABI rows are trustworthy C-vs-Rust codegen. The `mat33` rows are not: in `MatMul3x3`
nearly half the C path is marshalling, and the timed loop's whole pre-call body is `ldrd`/`strd`
copying two matrices onto the stack.

## Cross-language LTO (`xlto` profile)

`cargo make bench-stm32-xlto`, `cargo make bench-rp2350-xlto`. clang compiles the C to bitcode, rustc
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

Measured on stm32 in CI (`results.csv`, pipeline 486019), median cycles per call across the full
~26-input sweep per task. These numbers replace the one-session table this section used to carry.

| Task              | `lto` cfw | `xlto` cfw | `lto` glam | `xlto` glam | ratio           |
| ----------------- | --------- | ---------- | ---------- | ----------- | --------------- |
| `MatMul3x3`       | 349.6     | 146.9      | 126.5      | 126.4       | 2.76x -> 1.16x  |
| `QuatToRotMatrix` | 106.2     | 62.0       | 60.4       | 51.0        | 1.76x -> 1.22x  |
| `MatVecMul3x3`    | 97.3      | 100.0      | 56.3       | 52.7        | 1.73x -> 1.90x  |
| `CrossProduct`    | 41.8      | 52.6       | 35.3       | 36.2        | 1.18x -> 1.45x  |

`xlto` closes the two rows it can inline and widens the rest, because Rust gets inlined under the
same regime while the C stays behind a call. `CrossProduct` shows a second effect: `crazyflie-fw`'s
own call site gets 26% slower under `xlto`, not just relatively behind a faster glam. Read each row
at the profile where its wrapper can actually be inlined, and say which one you used.

### `xlto` is not a strict win

Across every stm32 task x library pair that ran under both `lto` and `xlto` (60 pairs), 20
improved by more than 3%, 14 regressed by more than 3%, and 26 were unchanged. The regressions
are not confined to C wrappers that fail to inline. Pure-Rust rows regress too, so part of this is
`xlto`'s `release`-based ThinLTO codegen itself, not only the FFI story:

| Task            | Library                        | `lto` -> `xlto` |
| ---------------- | ------------------------------ | ---------------- |
| `SinCos`         | micromath                      | +31%              |
| `QuatSlerp`      | crazyflie-fw                    | +28%              |
| `MatMul9x9`      | nalgebra                        | +20%              |
| `QuatMul`/`UnitQuatMul` | crazyflie-fw              | +14-15%           |
| `LeeController`  | glam / nalgebra / micromath     | +5-12%            |

Against that, the big wins outside the table above are `EkfStep` crazyflie-fw (2.06x) and
CMSIS-DSP's own `MatMul9x9` (1.88x) and `DotProduct64D` (1.78x). Both call real library routines
with pointer ABI, so they benefit from the same ThinLTO without any struct-marshalling story.

**Verdict:** `xlto` is not a blanket replacement for `lto`, and skipping it is not a mistake by
default. Build it for code shaped like `MatMul3x3`/`QuatToRotMatrix` (mat33 by-value ABI) or
CMSIS-DSP's larger linalg routines, where it is a clear win. Elsewhere treat it as close to a coin
flip: reach for it only when profiling shows the workload sits in a row it actually helps, not as
a default swap-in for `lto`.

Swapping gcc for clang is worth 1.05x (`CrossProduct`) to 1.33x (`MatMul3x3`) on its own, measured
at `lto` with no cross-LTO. gcc was never a deliberate choice; it is the `cc` crate's default for
`thumb*-none-eabi*`.
