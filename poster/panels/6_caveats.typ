#import "../theme.typ": accent

#let caveat(title, body) = block(width: 100%)[
  #text(weight: 800, size: 21pt)[#title]
  #v(5pt)
  #text(size: 19pt)[#body]
]

#let caveats-panel() = block(width: 100%, fill: rgb("#fbe9e6"), inset: (x: 28pt, y: 22pt))[
  #grid(
    columns: (10.5cm, 1fr, 1fr, 1fr),
    column-gutter: 30pt,
    align: (left + horizon, left + top, left + top, left + top),
    [
      #text(fill: accent, weight: 900, size: 28pt, tracking: 0.02em)[#upper("where it") \ #upper("reverses")]
      #v(6pt)
      #text(size: 17pt, fill: rgb("#666"))[Nuances and counter-examples across the benchmark grid]
    ],
    caveat("xlto is not always a win", [
      Of 60 task and library pairs on the STM32, `xlto` made 20 faster, 17 slower and left 23 unchanged (within 3%). micromath's SinCos got 31% slower.
    ]),
    caveat("Rust libm wins 3 of 5", [
      Rust libm is faster than newlib on exp (1.85x), ln (1.78x) and atan2 (1.33x). Which library you link decides each function, not the language.
    ]),
    caveat("Building for size favors C", [
      At `size`, C is 1.59x faster on MatMul9x9 and 1.56x on MatInverse9x9. Optimizing for size drops the loop unrolling nalgebra relies on.
    ]),
  )
]
