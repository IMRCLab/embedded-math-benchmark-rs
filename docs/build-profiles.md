# Build profiles

Four profiles ship, and `profile` is a CSV column. `release` is opt-level 3. `lto` adds fat LTO
and `codegen-units=1`. `size` is `opt-level="z"` on top of LTO. `xlto` compiles the C to bitcode
with clang and runs one ThinLTO over C and Rust together. Every profile sets `panic = "abort"`.
CI builds and runs the first three on every board.

Which profile makes a given C-vs-Rust number valid depends on the task's ABI class, and there is
no single right answer. See
[task-categories.md](task-categories.md#which-profile-makes-a-c-vs-rust-number-valid).

## Cross-language LTO (`xlto`)

`cargo make bench-stm32-xlto`, `cargo make bench-rp2350-xlto`; ARM hard-float only, and it needs
`clang` on `PATH`. clang compiles the C to bitcode, rustc emits bitcode under
`-Clinker-plugin-lto`, and rust-lld runs one ThinLTO over both. The profile is based on `release`,
not `lto`: fat LTO runs inside rustc and the C bitcode never joins that link, so cross-LTO there
silently does nothing. clang's LLVM major need not match `rustc -vV`'s, since newer LLVM reads
older bitcode.

Three things must line up or the C stays behind a call:

- **`-C target-cpu` must match clang's `-mcpu`**, or `ARMTTIImpl::areInlineCompatible` rejects the
  callee on target features.
- **`-import-instr-limit`** defaults to 100, too low for the bigger wrappers; we pass 500. Below
  that the inliner reports `NoDefinition` and never sees a body.
- **The LLVM function types must match.** rustc lowers a by-value HFA to `[3 x float]` /
  `[4 x float]`; clang keeps `%struct.vec` / `%struct.quat`. `InlineFunction` rejects the mismatch
  before cost analysis, so no remark is emitted and `alwaysinline` cannot override it. `ptr`,
  `float` and `[9 x i32]` params agree, and `sret` is an attribute rather than part of the type.

Most wrappers pass or return a `vec`/`quat` by value and hit the third rule. Only
`cf_quat2rotmat` was cheap to fix, by scalarizing its quat argument into four `float`s; the rest
mismatch on the *return*, which only an out-pointer would fix, and that would cost the other
profiles real memory traffic.

On stm32 this closes the two rows it can inline, `MatMul3x3` from 2.76x to 1.16x and
`QuatToRotMatrix` from 1.76x to 1.22x, and widens the rest, because Rust gets inlined under the
same regime while the C stays behind a call. `CrossProduct` shows a second effect: `crazyflie-fw`'s
own call site gets 26% slower under `xlto`, not just relatively behind a faster glam. This is
ThinLTO's own codegen cost, not the gcc-to-clang swap: the fat-LTO spike below holds clang and
cross-language joining fixed and only flips thin to fat, which recovers nearly all of it (1315 ->
1046 cycles, against 1044 at plain `lto`/gcc).

## `xlto` is not a strict win

Across every stm32 task x library pair that ran under both `lto` and `xlto` (60 pairs), 20
improved by more than 3%, 14 regressed by more than 3%, and 26 were unchanged. The regressions are
not confined to C wrappers that fail to inline. Pure-Rust rows regress too, so part of this is
`xlto`'s `release`-based ThinLTO codegen itself, not only the FFI story:

| Task                    | Library                     | `lto` -> `xlto` |
| ----------------------- | --------------------------- | --------------- |
| `SinCos`                | micromath                   | +31%            |
| `QuatSlerp`             | crazyflie-fw                | +28%            |
| `MatMul9x9`             | nalgebra                    | +20%            |
| `QuatMul`/`UnitQuatMul` | crazyflie-fw                | +14-15%         |
| `LeeController`         | glam / nalgebra / micromath | +5-12%          |

Against that, the big wins outside the table above are `EkfStep` crazyflie-fw (2.06x) and
CMSIS-DSP's own `MatMul9x9` (1.88x) and `DotProduct64D` (1.78x). Both call real library routines
with pointer ABI, so they benefit from the same ThinLTO without any struct-marshalling story.

**Verdict:** `xlto` is not a blanket replacement for `lto`, and skipping it is not a mistake by
default. Build it for code shaped like `MatMul3x3`/`QuatToRotMatrix` (mat33 by-value ABI) or
CMSIS-DSP's larger linalg routines, where it is a clear win. Elsewhere treat it as close to a coin
flip: reach for it only when profiling shows the workload sits in a row it actually helps.

Swapping gcc for clang is worth 1.05x (`CrossProduct`) to 1.33x (`MatMul3x3`) on its own, measured
at `lto` with no cross-LTO. gcc was never a deliberate choice; it is the `cc` crate's default for
`thumb*-none-eabi*`.

## Fat LTO spike (not merged)

Swapping `-flto=thin`/`lto` unset for `-flto`/`lto = "fat"` on stm32: broad win (22/60 pairs >3%
faster, 10 regressed) but `MatInverse9x9`/nalgebra, `EkfStep`/crazyflie-fw and `MatMul9x9`/cmsis-dsp
roughly double. Cause: fat's inliner has no importer budget, so it fully merges big routines
(`kalmanCorePredict`, `cmsis_matmul_9x9`) into their caller and blows past Cortex-M4's 8
D-registers. `cf_ekf_step` spill/reload instructions go 16 -> 189, despite total binary text
shrinking 5.6%. Not merged; see the `xlto-fat-lto-spike` worktree.

`CrossProduct`/crazyflie-fw is one of the wins, and the one that matters for the 26% figure above:
1315 cycles under thin drops to 1046 under fat, clang held fixed both times, landing almost exactly
on the 1044 cycles measured at plain `lto`/gcc. That is what isolates the regression to ThinLTO's
own codegen rather than the compiler swap.
