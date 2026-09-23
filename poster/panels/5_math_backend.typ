#import "../theme.typ": accent, accent-note, caption, panel, scaled-bars, sec-heading, subheading

#let sincos-rows = (
  ("Rust libm", 21802, "21,802", accent),
  ("newlib (C)", 2154, "2,154", black),
  ("micromath (Rust)", 373, "373", rgb("#bbb")),
)
#let sqrt-rows = (
  ("Rust libm", 423, "423", accent),
  ("newlib (C)", 233, "233", black),
  ("micromath (Rust)", 209, "209", rgb("#bbb")),
)
#let exp-rows = (
  ("Rust libm", 598, "598", accent),
  ("newlib (C)", 1106, "1,106", black),
  ("micromath (Rust)", 696, "696", rgb("#bbb")),
)

#let bar-group(title, verdict, rows) = block(width: 100%)[
  #grid(
    columns: (1fr, auto),
    text(weight: 700, size: 21pt)[#title], text(weight: 900, size: 21pt)[#verdict],
  )
  #v(4pt)
  #scaled-bars(rows, width: 14, bar-h: 0.58, gap: 0.2)
]

#let math-backend-panel() = panel(sec-heading(
  "02",
  "The linked math library opens a 10x gap",
  caption: caption[STM32F405 #sym.dot.c `lto` profile #sym.dot.c median ns],
))[
  #text(size: 26pt)[
    `sqrt`, `sin`/`cos` and `exp` are not in Rust's core library, so embedded Rust crates get them from
    the Rust `libm` crate. Against C's newlib it is up to #text(fill: accent, weight: 700)[10x] slower
    on sin/cos, but faster on exp, ln and atan2.
  ]
  #v(16pt)

  #grid(
    columns: (1fr, 1.1fr, 1.15fr),
    column-gutter: 30pt,
    [
      #bar-group("SinCos", [C #text(fill: accent)[10.1x] faster], sincos-rows)
      #v(10pt)
      #bar-group("Sqrt", [C #text(fill: accent)[1.82x] faster], sqrt-rows)
      #v(10pt)
      #bar-group("Exp", [Rust #text(fill: accent)[1.85x] faster], exp-rows)
      #v(6pt)
      #caption[
        *Rust libm*: pure-Rust math crate that glam and nalgebra call for sqrt, sin, exp.
        *newlib*: the C math library shipped with the Arm GCC toolchain.
        *micromath*: Rust crate of fast approximations. Each group scaled to its slowest bar.
      ]
    ],
    [
      #subheading("Fast approximations cost accuracy")
      #text(size: 21pt)[
        micromath's sqrt is off by #text(fill: accent, weight: 700)[2.0%] on average and
        #text(fill: accent, weight: 700)[5.7%] at worst.
      ]
      #v(10pt)
      #grid(
        columns: (auto, 1fr),
        column-gutter: 16pt,
        row-gutter: 8pt,
        align: (right + horizon, left + horizon),
        text(fill: accent, weight: 900, size: 32pt)[120,517], text(size: 19pt)[ULP mean error, micromath sqrt],
        text(weight: 900, size: 32pt)[0.07], text(size: 19pt)[ULP, Rust libm and newlib (identical results)],
      )
      #v(10pt)
      #block(width: 100%, fill: rgb("#f2f2f2"), inset: 14pt)[
        #text(size: 18pt)[
          *ULP* (unit in the last place): the step between two neighboring f32 values. 0 ULP is the exact
          answer; plain rounding stays within 1 to 2.
        ]
      ]
      #v(10pt)
      #text(size: 21pt)[Linear algebra is accurate in both languages: every C and Rust library stays within 15 ULP on average.]
    ],
    [
      #subheading("Why Rust libm loses on sin/cos and sqrt")
      #text(size: 21pt)[
        *sin/cos:* Rust libm computes in 64-bit doubles. This FPU only handles 32-bit floats, so every
        double operation runs in software. That is the whole 10x.
      ]
      #v(8pt)
      #text(size: 21pt)[
        *sqrt:* the chip has a sqrt instruction and newlib uses it. Rust libm has no 32-bit ARM path to it
        and falls back to a table lookup plus integer iterations.
      ]
      #v(10pt)
      #accent-note[
        #text(size: 21pt)[
          Composite operations inherit this. In `QuatSlerp` (acos plus three sines), cmath3d is
          #text(fill: accent, weight: 700)[10.3x] faster than glam. nalgebra swaps one sine for a sqrt
          and gets that down to #text(fill: accent, weight: 700)[6.8x].
        ]
      ]
    ],
  )
  #v(12pt)
  #text(size: 20pt)[
    *Other chips:* C's sqrt is 2.24x faster on the Cortex-M33 (RP2350). The ESP32-S3 links no C math
    library; there micromath's sqrt is 2.72x faster than Rust libm's. The RP2040 has no FPU, and its C
    build ends up calling a Rust software routine instead of newlib, so it gives no C-vs-Rust number.
  ]
]
