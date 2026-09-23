#import "../theme.typ": accent, panel, sec-heading

#let statements = (
  ("01", [In our tests, glam and nalgebra kept pace with C on linear algebra when both were built fairly.]),
  ("02", [Wrapping C that takes 3x3 matrices by value? Build with cross-language LTO, or pass pointers.]),
  ("03", [Check which library provides sqrt and sin/cos. On a 32-bit FPU, Rust libm's sin/cos is 10x slower than newlib.]),
  ("04", [Use fast-math crates like micromath only if your error budget allows percent-level error.]),
)

#let numbered-statement(num, body) = grid(
  columns: (auto, 1fr),
  column-gutter: 14pt,
  align: (left + top, left + top),
  text(fill: accent, weight: 900, size: 30pt)[#num],
  text(size: 22pt)[#body],
)

#let guidance-panel() = panel(sec-heading("04", "Early takeaways"))[
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    column-gutter: 34pt,
    ..statements.map(s => numbered-statement(s.at(0), s.at(1))),
  )
]
