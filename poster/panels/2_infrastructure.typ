#import "../theme.typ": accent, stat-block, subheading

#let infrastructure-panel() = block(width: 100%)[
  #subheading("open hardware-in-the-loop bench", color: accent)
  #grid(
    columns: (1fr, 1.25fr, 1.25fr, 1.2fr, 1.1fr),
    column-gutter: 30pt,
    stat-block("18 math primitives", [909 inputs baked into flash at compile time. Only the math call is timed, in on-chip clock cycles.]),
    stat-block("4 Rust libraries", [glam, nalgebra (linear algebra) #sym.dot.c micromath (fast approximations) #sym.dot.c libm (sqrt, sin, exp without an OS)]),
    stat-block("2 C libraries", [CMSIS-DSP (Arm's signal-processing library) #sym.dot.c cmath3d (the Crazyflie drone firmware's 3D math). C's sqrt and sin come from newlib.]),
    stat-block("5 chips", [STM32F405, nRF52840 (Cortex-M4F) #sym.dot.c RP2350 (M33) #sym.dot.c RP2040 (M0+, no FPU) #sym.dot.c ESP32-S3 (Xtensa)]),
    stat-block("4 build profiles", [`release`, `lto` (all Rust together), `xlto` (Rust and C together), `size`]),
  )
]
