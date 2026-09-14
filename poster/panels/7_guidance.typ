#import "../theme.typ": accent

#let statements = (
  ("01", [A C-versus-Rust number is tied to the build profile that produced it.]),
  ("02", [A slow sqrt or sin/cos in Rust points at the linked math backend, not the language.]),
  ("03", [Fast-math crates trade orders of magnitude of accuracy for a few times the speed.]),
)

#let numbered-statement(num, body) = grid(
  columns: (auto, 1fr),
  column-gutter: 14pt,
  align: (left + top, left + top),
  text(fill: accent, weight: 900, size: 30pt)[#num],
  text(size: 22pt)[#body],
)

#let guidance-panel() = block(width: 100%)[
  #text(weight: 800, size: 30pt, tracking: 0.03em)[IF YOU ARE CHOOSING A MATH LIBRARY TODAY]
  #v(14pt)
  #grid(
    columns: (1fr, 1fr, 1fr),
    column-gutter: 34pt,
    ..statements.map(s => numbered-statement(s.at(0), s.at(1))),
  )
]
