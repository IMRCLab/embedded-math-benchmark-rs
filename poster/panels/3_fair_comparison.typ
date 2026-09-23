#import "../theme.typ": accent, accent-note, caption, diverging-bars, diverging-lollipop, panel, profile-pill, sec-heading, subheading

// Paper data (STM32F405): C time / fastest of glam and nalgebra.
#let fair-rows = (
  ("DotProduct64D", 0.94),
  ("MatMul9x9", 0.98),
  ("UnitQuatMul", 1.07),
  ("QuatMul", 1.11),
  ("MatMul3x3", 1.16),
  ("CrossProduct", 1.19),
  ("QuatToRotMatrix", 1.22),
  ("RotateVector", 1.27),
  ("MatInverse9x9", 1.47),
)

// Paper Fig. 2.
#let groups = ("release", "lto", "xlto", "size")
#let series = (
  ("CrossProduct", rgb("#1a1a1a"), (2.70, 1.19, 1.45, 1.11)),
  ("MatMul3x3", rgb("#888"), (1.97, 2.76, 1.16, 1.45)),
  ("MatMul9x9", rgb("#cfcfcf"), (1.99, 2.21, 0.98, 0.63)),
  ("DotProduct64D", white, (1.41, 1.37, 0.94, 1.53)),
)
#let sublabels = (
  "each crate on its own",
  "all Rust together, C apart",
  "Rust and C together",
  "smallest binary",
)

#let legend() = grid(
  columns: 4 * (auto,),
  column-gutter: 20pt,
  align: horizon,
  ..series.map(s => grid(
    columns: (auto, auto),
    column-gutter: 8pt,
    align: horizon,
    box(width: 15pt, height: 15pt, fill: s.at(1), stroke: 0.6pt + black), text(size: 17pt)[#s.at(0)],
  )),
)

#let kind-card(title, tasks, body, profile) = [
  #grid(
    columns: (1fr, auto),
    align: (left + horizon, right + horizon),
    text(weight: 800, size: 21pt)[#title], [#text(size: 15pt, fill: rgb("#666"))[fairest at] #profile-pill(profile)],
  )
  #v(2pt)
  #text(size: 16pt, fill: rgb("#666"))[#tasks]
  #v(8pt)
  #text(size: 19pt)[#body]
]

#let fair-comparison-panel() = panel(sec-heading(
  "01",
  "Linear algebra: near parity when built fairly",
  caption: caption[STM32F405 #sym.dot.c Cortex-M4F, 168 MHz],
))[
  #text(size: 26pt)[
    With the wrong build profile, either language wins by up to #text(fill: accent, weight: 700)[2.8x].
    With the fair one, C is at most #text(fill: accent, weight: 700)[1.06x] faster and Rust at most
    #text(fill: accent, weight: 700)[1.47x] faster.
  ]
  #v(16pt)

  #subheading("The fairest build profile depends on the operand")
  #grid(
    columns: (1fr, 1fr, 1fr),
    column-gutter: 20pt,
    inset: (x: 18pt, top: 18pt, bottom: 26pt),
    stroke: 0.8pt + rgb("#ccc"),
    kind-card(
      "A few floats",
      "vectors, quaternions: CrossProduct, QuatMul, UnitQuatMul, RotateVector",
      [Passed in registers, so calling C costs little. `xlto` would make C's own code 26% slower, so `lto` is fairer.],
      "lto",
    ),
    kind-card(
      "A 3x3 matrix, by value",
      "MatMul3x3, QuatToRotMatrix",
      [Too big for registers: C copies nine floats through memory on every call, while Rust inlines them away. Only `xlto` lets C inline too.],
      "xlto",
    ),
    kind-card(
      "A pointer to a big buffer",
      "9x9 and 64-D: MatMul9x9, MatInverse9x9, DotProduct64D",
      [Nothing is copied. But under `lto` only the Rust side is optimized across files, so `xlto` is fair.],
      "xlto",
    ),
  )
  #v(18pt)
  #text(size: 21pt)[Every ratio below compares the faster C library (cmath3d or CMSIS-DSP) with the faster Rust crate (glam or nalgebra).]
  #v(14pt)

  #grid(
    columns: (1.1fr, 1fr),
    column-gutter: 40pt,
    [
      #subheading("Any profile: the winner flips")
      #align(center)[#diverging-bars(groups, series, up: 9, bar-w: 1.25, gap: 0.35, captions: sublabels)]
      #v(6pt)
      #align(center)[#legend()]
      #v(10pt)
      #accent-note[
        #text(size: 21pt)[
          Same 9x9 matrix multiply, same inputs: Rust #text(fill: accent, weight: 700)[2.21x] faster at `lto`,
          C #text(fill: accent, weight: 700)[1.59x] faster at `size`.
        ]
      ]
    ],
    [
      #subheading("Fairest profile: a tie")
      #v(6pt)
      #diverging-lollipop(fair-rows, domain: (0.85, 1.6), width: 24, row-h: 1.2)
      #v(8pt)
      #caption[Log scale. Gray band and gray dots: within 1.1x, a tie.]
      #v(14pt)
      #text(size: 21pt)[
        *Other chips agree.* On CrossProduct, Rust is 1.07x to 1.50x faster on Cortex-M0+, M33 and M4F,
        and glam and nalgebra are tied on every chip.
      ]
    ],
  )
]
