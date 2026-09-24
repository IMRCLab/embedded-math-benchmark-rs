#import "../theme.typ": accent, profile-pill, stat-block, subheading

#let profile-def(name, body) = grid(
  columns: (auto, 1fr),
  column-gutter: 12pt,
  align: (left + top, left + top),
  profile-pill(name), text(size: 19pt)[#body],
)

#let infrastructure-panel() = block(width: 100%)[
  #subheading("open hardware-in-the-loop bench", color: accent, caption: [All cycle counts from automated hardware-in-the-loop CI runs])
  #grid(
    columns: (1fr, 1.25fr, 1.25fr, 1.2fr),
    column-gutter: 30pt,
    stat-block("18 math primitives", [909 inputs baked into flash at compile time. Only the math call is timed, in on-chip clock cycles.]),
    stat-block("4 Rust libraries", [glam, nalgebra (linear algebra) #sym.dot.c micromath (fast approximations) #sym.dot.c libm (sqrt, sin, exp, which Rust's core library lacks)]),
    stat-block("2 C libraries", [CMSIS-DSP (Arm's signal-processing library) #sym.dot.c cmath3d (the Crazyflie drone firmware's 3D math). C's sqrt and sin come from newlib.]),
    stat-block("5 chips", [STM32F405, nRF52840 (Cortex-M4F) #sym.dot.c RP2350 (M33) #sym.dot.c RP2040 (M0+, no FPU) #sym.dot.c ESP32-S3 (Xtensa)]),
  )
  #v(16pt)
  #text(fill: accent, weight: 800, size: 16pt, tracking: 0.03em)[#upper("4 build profiles")]
  #h(10pt)
  #text(size: 19pt)[C code is always called from Rust through FFI, as when firmware wraps a C library. What the optimizer can see across that call decides the result.]
  #v(10pt)
  #grid(
    columns: (1fr, 1.25fr, 1.25fr, 1.2fr),
    column-gutter: 30pt,
    profile-def("release", [Each crate optimized on its own; C code is built with GCC.]),
    profile-def("lto", [All Rust optimized together (LTO); GCC-compiled C code remains an opaque call boundary.]),
    profile-def("xlto", [C compiled with Clang, followed by ThinLTO across Rust and C so calls can be inlined.]),
    profile-def("size", [Optimized for the smallest binary: Rust `opt-level=z`, C `-Os`.]),
  )
]
