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

#let bar-group(title, mult, rows) = block(width: 100%)[
  #grid(
    columns: (1fr, auto),
    text(weight: 700, size: 22pt)[#title], text(fill: accent, weight: 900, size: 26pt)[#mult],
  )
  #v(6pt)
  #scaled-bars(rows, width: 18)
]

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
      #subheading("The accuracy tradeoff of micromath")
      #text(fill: accent, weight: 900, size: 54pt)[120,517]
      #text(size: 19pt)[ mean ULP error on micromath's Sqrt]
      #v(14pt)
      #text(size: 21pt)[
        For our inputs, this leads to a mean relative error
        of #text(fill: accent, weight: 700)[2%] and a worst case of #text(fill: accent, weight: 700)[5.7%],
        whereas Rust libm and C newlib are a bit-exact match.
      ]
      #v(10pt)
      #accent-note[
        #text(size: 21pt)[
          That may sit inside the tolerance of a high-rate attitude loop that averages per-step noise away,
          but it is an explicit engineering tradeoff.
        ]
      ]
    ],
    [
      #subheading("Why Rust's libm is so slow")
      #text(size: 22pt)[
        C newlib works in f32, which the FPU runs in hardware. Rust libm always uses f64, which it has to emulate in software.
      ]
      #v(10pt)
      #accent-note[
        #text(size: 22pt)[
          Other tests inherit this, e.g. `QuatSlerp` calls sine and cosine three times, so
          cmath3d runs #text(fill: accent, weight: 700)[10.35x] faster than glam.
        ]
      ]
    ],
  )
]
