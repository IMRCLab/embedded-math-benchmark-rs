# Task categories

What each task measures, and therefore which comparisons it supports. Check here before
quoting a number as a C-vs-Rust result.

| Task | Category | Why |
|---|---|---|
| `CrossProduct` | arithmetic | `vcross`, pure mul/sub |
| `MatMul3x3` | arithmetic | `mmul`, triple loop |
| `MatVecMul3x3` | arithmetic | `mvmul` |
| `QuatMul` | arithmetic | `qqmul` |
| `UnitQuatMul` | arithmetic | `qqmul` |
| `QuatToRotMatrix` | arithmetic | `quat2rotmat` |
| `RotateVector` | arithmetic | `qvrot`, built from `vdot`/`vcross` |
| `MatInverse3x3` | arithmetic | Rust-only; source of the `ERROR` rows (near-singular inputs) |
| `Vec3Normalize` | libm-bound | `vnormalize` -> `vmag` -> `sqrtf` |
| `QuatSlerp` | libm-bound | `qslerp` -> `acosf` + three `sinf` |
| `Sqrt` `SinCos` `Atan2` `Exp` `Ln` | transcendental | direct libm entry points |
| `DotProduct64D` `MatMul9x9` `MatInverse9x9` | linalg | 9x9 / 64-element operands |
| `EkfStep` | composite | `kalman_core.c`: CMSIS `arm_mat_*` plus 18 `powf` |
| `LeeController` | composite | `controller_lee.c`: many primitives plus `sinf`/`cosf` |

## What each category supports

**arithmetic**: C-vs-Rust codegen, and library-vs-library. Caveat: `cf_math_wrapper.c` exists
only to expose `math3d.h`'s `static inline` functions as linkable symbols, so C pays a call the
real firmware inlines away. Treat any Rust margin as an upper bound.

**libm-bound**: nothing about language codegen. The result reflects which libm is linked, so
read it next to the transcendental rows.

**transcendental**: libm implementations against each other, newlib vs the Rust `libm` crate vs
micromath. The `crazyflie-fw` rows here hold no Crazyflie code; they are the ARM toolchain's
newlib. In results predating `84d6028` they are not even that, because `compiler_builtins`'
weak symbols shadowed the real `libm.a`. See [platforms.md](platforms.md).

**linalg**: the cleanest C-vs-Rust comparison in the suite. CMSIS-DSP routines are non-inline
library calls in real use too, so the FFI boundary is representative rather than an artifact.

**composite**: what a realistic control-loop step costs per platform. Not a language
comparison, because the C and Rust paths are different algorithms rather than two translations
of one. The ULP data shows this directly.
