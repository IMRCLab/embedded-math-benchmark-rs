#import "../theme.typ": accent, accent-note, caption, lollipop-chart, panel, sec-heading, subheading

#let parity-rows = (
  ("DotProduct64D", 0.94, "0.94", true),
  ("MatMul9x9", 0.98, "0.98", true),
  ("UnitQuatMul", 1.06, "1.06", false),
  ("QuatMul", 1.11, "1.11", false),
  ("MatMul3x3", 1.16, "1.16", false),
  ("CrossProduct", 1.19, "1.19", false),
  ("QuatToRotMatrix", 1.22, "1.22", false),
  ("RotateVector", 1.27, "1.27", false),
  ("MatInverse9x9", 1.47, "1.47", false),
)

#let legend() = grid(
  columns: (auto, auto, auto, auto, auto),
  column-gutter: 12pt,
  align: horizon,
  box(width: 15pt, height: 15pt, fill: black),
  text(size: 18pt)[Rust ahead],
  h(20pt),
  box(width: 15pt, height: 15pt, fill: accent),
  text(size: 18pt)[C ahead],
)

// S0-S3 ABI-class diagram: how many float members, and where they travel.
#let abi-class-row(name, desc, cells, note, note-fill: none) = grid(
  columns: (6.5cm, 1fr, 8.5cm),
  column-gutter: 14pt,
  align: (left + horizon, left + horizon, left + horizon),
  text(size: 20pt)[*#name* #desc],
  grid(
    columns: cells.len() * (1fr,),
    column-gutter: 4pt,
    ..cells.map(c => box(
      width: 100%,
      height: 1.7cm,
      fill: if note-fill != none { note-fill } else { rgb("#eee") },
      stroke: 0.8pt + rgb("#999"),
    )[#align(center + horizon)[#text(size: 19pt)[#c]]]),
  ),
  text(
    size: 18pt,
    fill: if note-fill != none { accent } else { rgb("#333") },
    weight: if note-fill != none { 700 } else { 400 },
  )[#note],
)

#let abi-classes-diagram() = block(width: 100%)[
  #grid(
    columns: (6.5cm, 1fr, 8.5cm),
    column-gutter: 14pt,
    [],
    grid(
      columns: (1fr, 1fr, 1fr, 1fr),
      column-gutter: 4pt,
      [S0], [S1], [S2], [S3],
    ),
    [],
  )
  #v(6pt)
  #abi-class-row("vec", [3 floats], ("x", "y", "z", ""), [in registers])
  #v(6pt)
  #abi-class-row("quat", [4 floats], ("w", "x", "y", "z"), [in registers])
  #v(6pt)
  #abi-class-row("mat33", [9 floats], ("nine members, five too many",), [to the stack], note-fill: rgb("#fbe3df"))
  #v(6pt)
  #abi-class-row("9x9 buf", [81 floats], ("one address, the data never moves",), [by pointer])
]

#let abi-shape-panel() = panel(sec-heading(
  "01",
  "The shape of the data costs more than the language",
  caption: caption[STM32F405 #sym.dot.c Cortex-M4F, 168 MHz],
))[
  #text(size: 26pt)[
    When we account for how the operands are passed, the actual difference stays small. All register- and pointer-passed primitives sit between #text(fill: accent, weight: 700)[0.94x] and #text(fill: accent, weight: 700)[1.47x].
  ]
  #v(16pt)

  #grid(
    columns: (1fr, 1fr),
    column-gutter: 40pt,
    [
      #lollipop-chart(parity-rows, domain: (0.85, 1.55), width: 26)
      #v(6pt)
      #legend()
      #v(4pt)
      #caption[C time / fastest Rust time, each read at the profile where it is valid]
    ],
    [
      #subheading("Why: the ABI sorts every task into three classes")
      #abi-classes-diagram()
      #v(12pt)
      #text(size: 22pt)[
        ARM hands a struct over in floating-point registers on one condition: every member is a float, and there
        are at most four of them. A 3x3 matrix is copied to the stack going in and returned through a hidden
        `sret` pointer coming out; traffic Rust never pays. Big operands cost nothing at the boundary, but the
        optimiser treats them differently again.
      ]
      #v(10pt)
      #accent-note[
        #text(size: 22pt)[
          These three classes are why the chart on the left cannot be read at one profile. Each class is a
          language comparison only where both sides get equivalent compiler treatment; that is *02*.
        ]
      ]
    ],
  )
]
