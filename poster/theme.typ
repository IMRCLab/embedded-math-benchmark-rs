// Shared colors, panel scaffolding and small chart primitives for the poster.
// Sized for A0 (see poster.typ) - font sizes and cetz `length` units here
// are roughly 1.4x the equivalent A2 reference poster's constants.

#let accent = rgb("#e03a1f")
#let accent-dark = rgb("#8c1f0f")
#let ink = rgb("#1a1a1a")
#let panel-gray = rgb("#e8e8e8")
#let rule-gray = rgb("#999")

// Numbered section heading with an optional right-aligned caption on the same row.
#let sec-heading(num, title, caption: none) = grid(
  columns: (1fr, auto),
  align: (left + horizon, right + horizon),
  [
    #box(fill: accent, inset: (x: 13pt, y: 8pt))[
      #text(fill: white, weight: 900, size: 34pt)[#num]
    ]
    #h(0.4em)
    #text(weight: 800, size: 34pt, tracking: 0.04em)[#upper(title)]
  ],
  if caption != none { caption } else { [] },
)

// A section panel: numbered heading + body.
#let panel(heading, body) = block(width: 100%)[
  #block(width: 100%, below: 24pt)[#heading]
  #body
]

// Full-width divider between major sections.
#let divider() = block(width: 100%, above: 24pt, below: 18pt)[
  #line(length: 100%, stroke: 2.4pt + black)
]

// Small caption/attribution line under a chart or panel.
#let caption(body) = text(size: 17pt, fill: rgb("#555"))[#body]

// Subsection label with a thin gray rule underneath, e.g. a "WHY: ..." blurb.
#let subheading(title, color: ink) = block(width: 100%, below: 12pt)[
  #text(fill: color, weight: 800, size: 22pt, tracking: 0.03em)[#upper(title)]
  #v(4pt)
  #line(length: 100%, stroke: 0.8pt + rgb("#aaa"))
]

// Paragraph with a left accent bar, for the one sentence worth setting apart.
#let accent-note(body) = block(
  width: 100%,
  inset: (left: 16pt),
  stroke: (left: 4pt + accent),
)[#body]

// Colored hero banner: big headline stat on the right, supporting claim on
// the left. Used directly under the header for the paper's single loudest
// number ("1.5x" etc).
#let hero-banner(claim, stat, stat-note) = block(
  width: 100%,
  fill: accent,
  inset: 32pt,
)[
  #grid(
    columns: (1fr, 13cm),
    column-gutter: 40pt,
    align: (left + horizon, right + horizon),
    text(fill: white, weight: 800, size: 40pt)[#claim],
    grid(
      columns: (auto,),
      align: center,
      row-gutter: 14pt,
      text(fill: white, weight: 900, size: 84pt)[#stat],
      block(width: 11cm)[#text(fill: white, size: 17pt)[#stat-note]],
    ),
  )
]

// Small stat block used in a row (infrastructure panel): label above,
// bold value below.
#let stat-block(label, value) = block(width: 100%)[
  #text(fill: accent, weight: 800, size: 16pt, tracking: 0.03em)[#upper(label)]
  #v(3pt)
  #text(size: 19pt)[#value]
]

// Gray placeholder box, e.g. for an unresolved chart.
#let placeholder-box(w, h, label) = box(
  width: w,
  height: h,
  fill: panel-gray,
  stroke: 1pt + rule-gray,
)[
  #align(center + horizon)[
    #text(size: 15pt, fill: rgb("#777"), weight: 700, tracking: 0.03em)[#upper(label)]
  ]
]

#import "@preview/tiaoma:0.3.0"

// Real, scannable QR code linking to the repo (paper, code and full tables
// live there). White quiet-zone border since zint renders the modules
// edge-to-edge with no margin of their own.
#let qr-code(url, size: 9cm) = box(
  width: size, height: size, fill: white, stroke: 1pt + rule-gray, inset: 6pt,
)[#tiaoma.qrcode(url, width: 100%, height: 100%)]

// cetz-based chart primitives

#import "@preview/cetz:0.3.4": canvas, draw

// Horizontal lollipop (dot) chart. `rows` is an array of (label, value, str-label, highlight).
// `domain` is (min, max) for the x axis; `parity` draws a dashed vline at 1.0.
#let lollipop-chart(rows, domain: (0.8, 1.6), width: 24, parity: 1.0) = canvas(length: 1cm, {
  import draw: *
  let (lo, hi) = domain
  let n = rows.len()
  let row-h = 1.0
  let x-of(v) = (v - lo) / (hi - lo) * width

  // parity gridline
  line((x-of(parity), 0.4), (x-of(parity), -(n) * row-h - 0.3), stroke: (paint: rule-gray, dash: "dashed", thickness: 1.2pt))
  content((x-of(parity), 0.9), text(size: 15pt, fill: rule-gray)[#parity#sym.times])

  for (i, row) in rows.enumerate() {
    let (label, value, vlabel, hi-lit) = row
    let y = -i * row-h
    let col = if hi-lit { accent } else { black }
    content((-0.4, y), text(size: 16pt)[#label], anchor: "east")
    line((x-of(parity), y), (x-of(value), y), stroke: 1pt + rgb("#ccc"))
    circle((x-of(value), y), radius: 0.17, fill: col, stroke: none)
    content((x-of(value) + 0.5, y), text(size: 16pt, fill: col, weight: 700)[#vlabel], anchor: "west")
  }
})

// Grouped vertical bar chart. `groups` is an array of group-label; `series`
// is an array of (series-label, color, values-array-matching-groups).
// `captions`, if given, is one gray subcaption per group, drawn as a second
// line under the group name, kept inside the canvas (rather than a
// separate Typst grid below it) so it lines up with the bars regardless of
// how wide the surrounding container is.
#let bar-chart(groups, series, height: 10, bar-w: 0.85, gap: 0.3, parity: none, captions: none) = canvas(length: 1cm, {
  import draw: *
  let max-v = calc.max(..series.map(s => calc.max(..s.at(2))))
  let y-of(v) = v / max-v * height
  let group-w = series.len() * (bar-w + gap) + gap * 2

  if parity != none {
    line((-0.5, y-of(parity)), (groups.len() * group-w, y-of(parity)), stroke: (paint: rule-gray, dash: "dashed", thickness: 1.2pt))
    content((-0.9, y-of(parity)), text(size: 15pt, fill: rule-gray)[#parity], anchor: "east")
  }

  for (gi, gname) in groups.enumerate() {
    let x0 = gi * group-w
    for (si, s) in series.enumerate() {
      let (_, color, values) = s
      let v = values.at(gi)
      let x = x0 + gap + si * (bar-w + gap)
      rect((x, 0), (x + bar-w, y-of(v)), fill: color, stroke: 0.6pt + black)
      content((x + bar-w / 2, y-of(v) + 0.45), text(size: 15pt, weight: 700)[#v], anchor: "south")
    }
    content((x0 + group-w / 2, -0.6), text(size: 16pt, weight: 700)[#gname], anchor: "north")
    if captions != none {
      content((x0 + group-w / 2, -1.15), text(size: 13pt, fill: rule-gray)[#captions.at(gi)], anchor: "north")
    }
  }
})

// Single-group horizontal bars scaled to their own max (used for the
// SinCos / Sqrt provider comparisons - each block scaled independently).
#let scaled-bars(rows, width: 16, bar-h: 0.65, gap: 0.25) = canvas(length: 1cm, {
  import draw: *
  let max-v = calc.max(..rows.map(r => r.at(1)))
  let x-of(v) = v / max-v * width
  for (i, row) in rows.enumerate() {
    let (label, value, vlabel, color) = row
    let y = -i * (bar-h + gap)
    content((-0.4, y + bar-h / 2), text(size: 16pt)[#label], anchor: "east")
    rect((0, y), (x-of(value), y + bar-h), fill: color, stroke: 0.6pt + black)
    content((x-of(value) + 0.35, y + bar-h / 2), text(size: 16pt, weight: 700)[#vlabel], anchor: "west")
  }
})
