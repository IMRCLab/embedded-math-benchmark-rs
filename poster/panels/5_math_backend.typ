#import "../theme.typ": accent, accent-note, panel, sec-heading, caption, scaled-bars, placeholder-box, subheading

#let sincos-rows = (
  ("libm (Rust)", 21802, "21,802", accent),
  ("newlib (C)", 2154, "2,154", black),
  ("micromath", 373, "373", rgb("#bbb")),
)
#let sqrt-rows = (
  ("libm", 423, "423", accent),
  ("newlib", 233, "233", black),
  ("micromath", 209, "209", rgb("#bbb")),
)

#let bar-group(title, mult, rows) = block(width: 100%)[
  #grid(
    columns: (1fr, auto),
    text(weight: 700, size: 22pt)[#title],
    text(fill: accent, weight: 900, size: 26pt)[#mult],
  )
  #v(6pt)
  #scaled-bars(rows, width: 18)
]

// The speed-vs-accuracy scatter is log-log with dashed connector lines
// joining each fast-math routine to the exact one it replaces. Better
// rendered from real results.csv data via the existing viz/ pipeline than
// hand-drawn here. Swap this placeholder for that exported figure.
#let accuracy-scatter-placeholder() = placeholder-box(100%, 8cm, "speed-vs-accuracy scatter (viz/, log-log, dashed connectors)")

#let math-backend-panel() = panel(sec-heading(
  "03",
  "The math backend swings 10x",
  caption: caption[lto profile #sym.dot.c median ns],
))[
  #text(size: 26pt)[
    Same operation, three implementations, a #text(fill: accent, weight: 700)[10x] spread, and the gap
    tracks the library, not the language.
  ]
  #v(16pt)

  #grid(
    columns: (1fr, 1.3fr, 1fr),
    column-gutter: 30pt,
    [
      #bar-group("SinCos", [10.1#sym.times], sincos-rows)
      #v(6pt)
      #caption[one sine/cosine pair]
      #v(20pt)
      #bar-group("Sqrt", [1.82#sym.times], sqrt-rows)
      #v(6pt)
      #caption[one square root]
      #v(6pt)
      #caption[Each group is scaled to its own slowest bar. newlib is the ARM GCC toolchain's C math library.]
    ],
    [
      #subheading("And the fast route is paid for in accuracy")
      #text(fill: accent, weight: 900, size: 54pt)[120,517]
      #text(size: 19pt)[ mean ULP error on micromath's Sqrt, against 0.07 for the routine it replaces]
      #v(12pt)
      #accuracy-scatter-placeholder()
      #v(6pt)
      #caption[up and to the left is faster and less accurate; dashed lines join a fast-math routine to the one it replaces]
    ],
    [
      #subheading("Why: the FPU is 32 bits wide")
      #text(size: 22pt)[
        newlib works in f32, which the FPU runs in hardware. libm evaluates the same polynomials in f64, which
        it has to emulate in software. That is the whole 10x.
      ]
      #v(10pt)
      #text(size: 22pt)[
        Sqrt is the same story: newlib emits one `vsqrt.f32` instruction; libm has no 32-bit ARM backend at all.
      ]
      #v(10pt)
      #accent-note[
        #text(size: 22pt)[
          Real code inherits whichever way its function went. QuatSlerp calls sine and cosine three times, so
          cmath3d runs #text(fill: accent, weight: 700)[10.35x] faster than glam.
        ]
      ]
    ],
  )
]
