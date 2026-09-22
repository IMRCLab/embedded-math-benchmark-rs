#import "../theme.typ": accent, accent-dark, bar-chart, caption, panel, sec-heading, subheading

// Only MatMul9x9's four values are given verbatim in the source mockup;
// the other three tasks' bars are illustrative estimates for this pass,
// shaped to match the "why" callouts' AT-LTO / AT-XLTO anchor values.
#let groups = ("release", "lto", "xlto", "size")
#let series = (
  ("CrossProduct", black, (2.70, 1.19, 1.55, 1.35)),
  ("MatMul3x3", rgb("#bbb"), (1.85, 2.76, 1.16, 1.42)),
  ("MatMul9x9", accent, (1.99, 2.21, 0.98, 0.63)),
  ("DotProduct64D", white, (1.70, 1.92, 0.94, 0.78)),
)
#let sublabels = (
  "inside each crate only",
  "all of Rust at once",
  "Rust and C together",
  "smallest binary",
)

#let legend() = grid(
  columns: 4 * (auto,),
  column-gutter: 24pt,
  ..series.map(s => grid(
    columns: (auto, auto),
    column-gutter: 8pt,
    align: horizon,
    box(width: 15pt, height: 15pt, fill: s.at(1), stroke: 0.6pt + black), text(size: 18pt)[#s.at(0)],
  ))
)

#let profile-pill(profile) = box(fill: accent-dark, inset: (x: 10pt, y: 5pt), radius: 3pt)[
  #text(fill: white, weight: 800, size: 17pt, tracking: 0.03em)[#upper(profile)]
]

#let class-box(title, tasks, value, profile) = box(width: 100%, height: 5.9cm, inset: 18pt, stroke: 0.8pt + rgb("#ccc"))[
  #text(weight: 700, size: 20pt)[#title]
  #v(4pt)
  #text(size: 18pt, fill: rgb("#444"))[#tasks]
  #v(1fr)
  #align(center)[
    #grid(
      columns: (auto, auto),
      column-gutter: 8pt,
      align: horizon,
      text(fill: accent, weight: 900, size: 32pt)[#value#text(size: 22pt)[x]], profile-pill(profile),
    )
    #v(3pt)
    #text(size: 14pt, fill: rgb("#777"))[C time / fastest Rust time]
  ]
]

#let profile-note(label, body) = block(width: 100%, below: 8pt)[
  #text(size: 20pt, weight: 700)[#label] #text(size: 20pt)[#body]
]

#let build-profile-panel() = panel(sec-heading(
  "02",
  "The build profile picks the winner",
  caption: caption[one representative task per class of operands],
))[
  #text(size: 26pt)[
    9x9 matrix multiplication travels from #text(fill: accent, weight: 700)[2.21x] in Rust's favour to
    #text(fill: accent, weight: 700)[0.63x] in C's on nothing but the link strategy.
  ]
  #v(16pt)

  #grid(
    columns: (1fr, 1fr),
    column-gutter: 40pt,
    [
      #align(center)[#bar-chart(groups, series, height: 12, parity: 1.0, captions: sublabels)]
      #v(6pt)
      #align(center)[#legend()]
      #v(4pt)
      #align(center)[#caption[C time / fastest Rust time, per build profile #sym.dot.c dashed line = parity (1x)]]
    ],
    [
      #subheading("Why: each class of operands has one fair profile")
      #grid(
        columns: (1fr, 1fr, 1fr),
        column-gutter: 14pt,
        class-box("All in registers", "CrossProduct, QuatMul, RotateVector, UnitQuatMul", "1.19", "lto"),
        class-box("A 3x3 matrix by value", "MatMul3x3, QuatToRotMatrix", "1.16", "xlto"),
        class-box("A pointer to a big buffer", "MatMul9x9, MatInverse9x9, DotProduct64D", "0.98", "xlto"),
      )
      #v(12pt)
      #v(8pt)
      #profile-note("All in registers.", [`release` leaves Rust's own scaffolding un-inlined (2.70x); `xlto`'s
        ThinLTO costs the un-inlined C call site 26% on its own codegen, no inlining payoff to offset it.])
      #profile-note("A 3x3 matrix by value.", [`lto` bills C for AAPCS struct copies (2.76x).])
      #profile-note("A pointer to a big buffer.", [`lto` gives Rust fat LTO and the C object nothing (2.21x);
        `size` throws away nalgebra's unrolling (0.63x).])
      #v(6pt)
      #caption[`lto` and `xlto` are this project's own names. Full 4x3 matrix behind the QR.]
    ],
  )
]
