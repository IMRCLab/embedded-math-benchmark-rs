#import "../theme.typ": accent, accent-note, caption, panel, scaled-bars, sec-heading, subheading

#let sincos-rows = (
  ("libm (Rust)", 21802, "21,802", accent),
  ("newlib (C)", 2154, "2,154", black),
  ("micromath", 373, "373", rgb("#bbb")),
)
#let sqrt-rows = (
  ("libm (Rust)", 423, "423", accent),
  ("newlib (C)", 233, "233", black),
  ("micromath", 209, "209", rgb("#bbb")),
)
#let exp-rows = (
  ("libm (Rust)", 598, "598", accent),
  ("newlib (C)", 1106, "1,106", black),
  ("micromath", 696, "696", rgb("#bbb")),
)

#let bar-group(title, mult, rows, mult-color: accent) = block(width: 100%)[
  #grid(
    columns: (1fr, auto),
    text(weight: 700, size: 21pt)[#title], text(fill: mult-color, weight: 900, size: 24pt)[#mult],
  )
  #v(4pt)
  #scaled-bars(rows, width: 17, bar-h: 0.58, gap: 0.2)
]

#let math-backend-panel() = panel(sec-heading(
  "03",
  "Transcendental backend creates a 10x gap",
  caption: caption[lto profile #sym.dot.c median ns],
))[
  #text(size: 26pt)[
    Identical operations diverge by up to #text(fill: accent, weight: 700)[10x] across math backends. The
    performance gap tracks library architecture, not language syntax.
  ]
  #v(16pt)

  #grid(
    columns: (1fr, 1.25fr, 1.15fr),
    column-gutter: 26pt,
    [
      #bar-group("SinCos", [10.1#sym.times C], sincos-rows)
      #v(10pt)
      #bar-group("Sqrt", [1.82#sym.times C], sqrt-rows)
      #v(10pt)
      #bar-group("Exp", [1.85#sym.times Rust], exp-rows, mult-color: black)
      #v(4pt)
      #caption[Each group scaled to its own slowest bar. newlib is the standard C math runtime in arm-none-eabi-gcc.]
    ],
    [
      #subheading("Fast-math precision tradeoff")
      #text(fill: accent, weight: 900, size: 54pt)[120,517]
      #text(size: 19pt)[ mean ULP error on micromath's Sqrt]
      #v(14pt)
      #text(size: 21pt)[
        Yields a #text(fill: accent, weight: 700)[2.0%] mean and #text(fill: accent, weight: 700)[5.7%]
        peak relative error; Rust libm and C newlib achieve bit-exact parity (0.07 mean ULP).
      ]
      #v(10pt)
      #accent-note[
        #text(size: 21pt)[
          Tolerable for high-rate attitude estimation where sensor fusion averages per-step noise,
          but prohibitive for dead-reckoning or kinematic integration.
        ]
      ]
    ],
    [
      #subheading("Root cause: generic f64 fallback on a 32-bit FPU")
      #text(size: 21pt)[
        The microcontroller's FPU only accelerates single precision (f32) in hardware. C newlib provides
        target-optimized 32-bit routines. Rust's libm lacks ARM FPU specialization for these operations
        and falls back to a generic portable implementation in 64-bit float (f64), forcing costly
        software emulation on 32-bit hardware.
      ]
      #v(10pt)
      #accent-note[
        #text(size: 21pt)[
          Compound workloads inherit this: in `QuatSlerp`, cmath3d runs #text(fill: accent, weight: 700)[10.3x]
          faster than glam by evaluating three sines. nalgebra cuts C's lead to #text(fill: accent, weight: 700)[6.8x]
          by trading one sine for a fast sqrt.
        ]
      ]
    ],
  )
]
