#import "../theme.typ": accent, panel, sec-heading

#let statements = (
  ("Linear algebra", [Use glam or nalgebra: built fairly, they match C baselines across all tested platforms.]),
  (
    "FFI boundaries",
    [Avoid by-value copies for structs exceeding register capacity: use cross-language LTO (`xlto`) or pass by pointer.],
  ),
  (
    "Transcendental math",
    [Audit sqrt and trigonometric providers on single-precision FPUs to avoid soft-float emulation.],
  ),
  (
    "Approximations",
    [Restrict fast-math crates (`micromath`) to noise-tolerant control loops with validated error budgets.],
  ),
)

#let rule-statement(header, body) = block(width: 100%)[
  #text(fill: accent, weight: 800, size: 20pt, tracking: 0.02em)[#upper(header)]
  #v(3pt)
  #text(size: 18.5pt)[#body]
]

#let guidance-panel() = panel(sec-heading("04", "Key takeaways"))[
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    column-gutter: 34pt,
    ..statements.map(s => rule-statement(s.at(0), s.at(1))),
  )
]
