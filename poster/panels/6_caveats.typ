#import "../theme.typ": accent

#let caveat(title, body) = block(width: 100%)[
  #text(weight: 700, size: 21pt)[#title]
  #v(6pt)
  #text(size: 19pt)[#body]
]

#let caveats-panel() = block(width: 100%, fill: rgb("#fbe9e6"), inset: 26pt)[
  #grid(
    columns: (auto, 1fr, 1fr, 1fr),
    column-gutter: 30pt,
    align: (left + top, left + top, left + top, left + top),
    text(fill: accent, weight: 900, size: 27pt, tracking: 0.02em)[#upper("where it") \ #upper("reverses")],
    caveat("xlto is not a strict win", [
      Of 60 task x library pairs: 20 faster, 26 unchanged, 14 slower. Pure-Rust rows regress too:
      micromath SinCos +31%.
    ]),
    caveat("libm wins three of the five transcendentals", [
      Median ns, libm vs. newlib: Exp 598/1,106 #sym.dot.c Ln 566/1,007 #sym.dot.c Atan2 1,091/1,456. The f64
      penalty is not uniform.
    ]),
    caveat("Both directions are represented", [
      Two of the nine arithmetic primitives land below parity, and the 3x3-by-value class reaches 2.76x at
      the wrong profile. Nothing here is a clean sweep for either language.
    ]),
  )
]
