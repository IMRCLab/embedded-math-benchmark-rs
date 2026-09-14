#import "../theme.typ": accent

#let footer-block(label, body) = block(width: 100%)[
  #text(fill: accent, weight: 800, size: 19pt, tracking: 0.04em)[#upper(label)]
  #v(5pt)
  #text(size: 19pt)[#body]
]

#let footer-panel() = grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 34pt,
  footer-block("Running now", [
    Five boards run continuously in CI, including the nRF52840 added since the paper. The RP2350's
    RISC-V core is nearly ready.
  ]),
  footer-block("Next", [
    Closed-loop workloads: an EKF update and a geometric controller step, where ABI class, provider
    and accuracy tradeoff compound at once.
  ]),
  footer-block("Versions", [
    glam 0.30.10 #sym.dot.c nalgebra 0.35.0 #sym.dot.c micromath 2.1.0 #sym.dot.c libm 0.2.16 #sym.dot.c
    CMSIS-DSP 1.17.1 #sym.dot.c cmath3d 54f31e2. rustc 1.98.1, arm-none-eabi-gcc 14.2.1, clang 19.1.7.
  ]),
)
