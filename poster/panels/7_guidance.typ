#import "../theme.typ": accent, panel, sec-heading

#let statements = (
  ("Linear algebra", [Use glam or nalgebra with confidence: built fairly, they match C baselines across all tested platforms.]),
  ("FFI boundaries", [Eliminate 3#sym.times 3 struct copying via cross-language ThinLTO (`xlto`) or explicit pointer passing.]),
  ("Transcendental math", [Audit sqrt and trigonometric providers on single-precision FPUs to avoid soft-float emulation.]),
  ("Approximations", [Restrict fast-math crates (`micromath`) to noise-tolerant attitude loops with validated error budgets.]),
)

#let rule-statement(header, body) = block(width: 100%)[
  #text(fill: accent, weight: 800, size: 21pt, tracking: 0.02em)[#upper(header)]
  #v(6pt)
  #text(size: 20pt)[#body]
]

#let guidance-panel() = panel(sec-heading("04", "Key takeaways"))[
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    column-gutter: 34pt,
    ..statements.map(s => rule-statement(s.at(0), s.at(1))),
  )
]
