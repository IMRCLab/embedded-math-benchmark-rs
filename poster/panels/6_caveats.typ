#import "../theme.typ": accent

#let question(title, body) = block(width: 100%)[
  #text(weight: 800, size: 21pt)[#title]
  #v(5pt)
  #text(size: 19pt)[#body]
]

#let open-questions-panel() = block(width: 100%, fill: rgb("#fbe9e6"), inset: (x: 28pt, y: 22pt))[
  #grid(
    columns: (10.5cm, 1fr, 1fr, 1fr),
    column-gutter: 30pt,
    align: (left + horizon, left + top, left + top, left + top),
    [
      #text(fill: accent, weight: 900, size: 28pt, tracking: 0.02em)[#upper("open") \ #upper("questions")]
    ],
    question("One C compiler for every profile?", [
      `xlto` swaps gcc for clang as well as the linking. clang alone makes the C 1.05x to 1.33x faster at `lto`, so part of the `xlto` gain is the compiler.
    ]),
    question("Fat LTO instead of ThinLTO for xlto?", [
      `xlto` is not always a win: of 60 pairs on the STM32, 20 got faster, 17 slower. A fat-LTO prototype recovers the C's 26% loss on CrossProduct but doubles three large routines, which run out of registers.
    ]),
    question("Hypothesis: flash caching slows the RP2350's EKF step", [
      The RP2350 runs code from flash through a small cache, and the EKF step's large code keeps missing it. Built for `size`, the code shrinks and the RP2350 beats the STM32.
    ]),
  )
]
