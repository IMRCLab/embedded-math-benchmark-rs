# Build profiles

Four profiles ship, and `profile` is a CSV column. `release` is opt-level 3. `lto` adds fat LTO and `codegen-units=1`. `size` is `opt-level="z"` on top of LTO. `xlto` compiles the C to bitcode with clang and runs one ThinLTO over C and Rust together. Every profile sets `panic = "abort"`. CI runs `release`, `lto` and `size` on every board, and `xlto` on the hard-float ARM boards (stm32, nrf52840, rp2350-arm).

Which profile makes a C-vs-Rust number valid depends on the task's ABI class, see [task-categories.md](task-categories.md#which-profile-makes-a-c-vs-rust-number-valid).

## Cross-language LTO (`xlto`)

`cargo make bench-stm32-xlto`, `bench-nrf52840-xlto` and `bench-rp2350-xlto`; ARM hard-float only, and it needs `clang` on `PATH`. clang compiles the C to bitcode, rustc emits bitcode under `-Clinker-plugin-lto`, and rust-lld runs one ThinLTO over both. The profile is based on `release`, not `lto`: fat LTO runs inside rustc and the C bitcode never joins that link, so cross-LTO there silently does nothing. clang's LLVM major need not match `rustc -vV`'s, since newer LLVM reads older bitcode.

Three things must line up or the C stays behind a call:

- **`-C target-cpu` must match clang's `-mcpu`**, or `ARMTTIImpl::areInlineCompatible` rejects the callee on target features.
- **`-import-instr-limit`** defaults to 100, too low for the bigger wrappers; we pass 500. Below that the inliner reports `NoDefinition` and never sees a body.
- **The LLVM function types must match.** rustc lowers a by-value HFA to `[3 x float]` / `[4 x float]`; clang keeps `%struct.vec` / `%struct.quat`. `InlineFunction` rejects the mismatch before cost analysis, so no remark is emitted and `alwaysinline` cannot override it. `ptr`, `float` and `[9 x i32]` params agree, and `sret` is an attribute rather than part of the type.

Most wrappers pass or return a `vec`/`quat` by value and hit the third rule. Only `cf_quat2rotmat` was cheap to fix, by scalarizing its quat argument into four `float`s; the rest mismatch on the *return*, which only an out-pointer would fix, and that would cost the other profiles real memory traffic. So of the `math3d` wrappers only `MatMul3x3` and `QuatToRotMatrix` inline. The rest stay behind a call while the Rust side gets inlined.

## Caveats

- `xlto` is not a default upgrade over `lto`. Its `release`-based ThinLTO codegen regresses some rows, pure-Rust ones included, and even slows `crazyflie-fw`'s own `CrossProduct` call site. It clearly wins on the `mat33` by-value wrappers and CMSIS-DSP's larger routines.
- It also swaps gcc for clang, which alone speeds up the C side. gcc was never a deliberate choice; it is the `cc` crate's default for `thumb*-none-eabi*`.
- Fat LTO in place of ThinLTO was tried and not merged: it merges big routines such as `kalmanCorePredict` into their caller, which then spills floating-point registers. See branch `xlto-fat-lto`.
