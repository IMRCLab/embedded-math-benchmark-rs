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
    caveat("XLTO is not a universal win", [
      Across 60 benchmark pairs: 21 faster, 17 unchanged, 22 slower. Cross-language ThinLTO perturbs pure-Rust inlining thresholds, causing regressions like micromath SinCos (+31%).
    ]),
    caveat("libm leads 3 of 5 transcendentals", [
      The f64 penalty is not uniform: when generic series avoid heavy 64-bit emulation, Rust libm outperforms C newlib on Exp (1.85x), Ln (1.78x), and Atan2 (1.33x).
    ]),
    caveat("Soft-float flips the gap", [
      On Cortex-M0+ (RP2040, no hardware FPU), both languages run soft-float. Rust libm runs SinCos 14% faster than C (9.8k vs 11.3k cycles). The 10x penalty is strictly an FPU artifact.
    ]),
  )
]
