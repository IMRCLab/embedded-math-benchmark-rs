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
#let subheading(title, color: ink, caption: none) = block(width: 100%, below: 12pt)[
  #grid(
    columns: (1fr, auto),
    align: (left + bottom, right + bottom),
    text(fill: color, weight: 800, size: 22pt, tracking: 0.03em)[#upper(title)],
    if caption != none { text(size: 16pt, fill: rgb("#666"))[#caption] } else { [] },
  )
  #v(4pt)
  #line(length: 100%, stroke: 0.8pt + rgb("#aaa"))
]

// Paragraph with a left accent bar, for the one sentence worth setting apart.
#let accent-note(body) = block(
  width: 100%,
  inset: (left: 16pt, y: 8pt),
  stroke: (left: 4pt + accent),
)[#body]

// Colored hero banner: the question on top, the claim on the left, and a
// column of (value, label) stats on the right.
#let hero-banner(question, claim, stats) = block(
  width: 100%,
  fill: accent,
  inset: 32pt,
)[
  #grid(
    columns: (1fr, auto),
    column-gutter: 50pt,
    align: (left + horizon, left + horizon),
    [
      #text(fill: white, size: 26pt)[#question]
      #v(10pt)
      #text(fill: white, weight: 800, size: 40pt)[#claim]
    ],
    grid(
      columns: (auto, auto),
      column-gutter: 16pt,
      row-gutter: 10pt,
      align: (right + horizon, left + horizon),
      ..stats.map(((value, label)) => (
        text(fill: white, weight: 900, size: 46pt)[#value],
        text(fill: white, size: 21pt)[#label],
      )).flatten(),
    ),
  )
]

// Dark pill with a build profile name, e.g. XLTO.
#let profile-pill(profile) = box(fill: accent-dark, inset: (x: 10pt, y: 5pt), radius: 3pt)[
  #text(fill: white, weight: 800, size: 17pt, tracking: 0.03em)[#upper(profile)]
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

// Ratio as a fixed two-decimal string, e.g. 1.5 -> "1.50".
#let fmt2(x) = {
  let s = str(calc.round(x, digits: 2))
  if not s.contains(".") { s + ".00" } else if s.split(".").at(1).len() == 1 { s + "0" } else { s }
}

// How many times faster the winner is, given r = C time / Rust time.
#let speedup(r) = if r >= 1 { r } else { 1 / r }

// Horizontal dot chart of C/Rust time ratios on a log axis, so "C 1.1x faster"
// and "Rust 1.1x faster" sit at equal distance from parity. `rows` is an array of
// (label, ratio); dots inside +-`band` are drawn gray as a tie.
#let diverging-lollipop(rows, domain: (0.85, 1.6), width: 24, row-h: 1.0, band: 1.1) = canvas(length: 1cm, {
  import draw: *
  let (lo, hi) = domain
  let n = rows.len()
  let x-of(r) = (calc.ln(r) - calc.ln(lo)) / (calc.ln(hi) - calc.ln(lo)) * width
  let top = 0.55
  let bottom = -(n - 1) * row-h - 0.55

  rect((x-of(1 / band), top), (x-of(band), bottom), fill: rgb("#f0f0f0"), stroke: none)
  line((x-of(1), top), (x-of(1), bottom), stroke: (paint: rule-gray, dash: "dashed", thickness: 1.2pt))
  content((x-of(1) - 0.3, top + 0.45), text(size: 16pt, weight: 700)[C faster], anchor: "east")
  content((x-of(1) + 0.3, top + 0.45), text(size: 16pt, weight: 700)[Rust faster], anchor: "west")

  for (i, row) in rows.enumerate() {
    let (label, r) = row
    let y = -i * row-h
    let col = if speedup(r) < band { rgb("#999") } else { ink }
    content((-0.4, y), text(size: 16pt)[#label], anchor: "east")
    line((x-of(1), y), (x-of(r), y), stroke: 1pt + rgb("#bbb"))
    circle((x-of(r), y), radius: 0.17, fill: col, stroke: none)
    let side = if r >= 1 { 1 } else { -1 }
    content(
      (x-of(r) + side * 0.4, y),
      text(size: 16pt, fill: col, weight: 700)[#fmt2(speedup(r))#sym.times],
      anchor: if r >= 1 { "west" } else { "east" },
    )
  }
})

// Grouped vertical bars of C/Rust time ratios on a log axis around parity:
// up = Rust faster, down = C faster. `series` is an array of
// (label, fill, ratios-per-group). `captions` is one gray subcaption per group.
#let diverging-bars(groups, series, up: 8, max-ratio: 3.0, min-ratio: 0.6, bar-w: 0.85, gap: 0.3, captions: none) = canvas(length: 1cm, {
  import draw: *
  let y-of(r) = calc.ln(r) / calc.ln(max-ratio) * up
  let bottom = y-of(min-ratio)
  let group-w = series.len() * (bar-w + gap) + gap * 2
  let total-w = groups.len() * group-w

  line((-0.3, 0), (total-w, 0), stroke: (paint: rule-gray, dash: "dashed", thickness: 1.2pt))
  content((-0.5, y-of(max-ratio) * 0.85), text(size: 16pt, weight: 700)[Rust \ faster], anchor: "east")
  content((-0.5, bottom * 0.7), text(size: 16pt, weight: 700)[C \ faster], anchor: "east")

  for (gi, gname) in groups.enumerate() {
    let x0 = gi * group-w
    for (si, s) in series.enumerate() {
      let (_, color, values) = s
      let v = values.at(gi)
      let x = x0 + gap + si * (bar-w + gap)
      rect((x, 0), (x + bar-w, y-of(v)), fill: color, stroke: 0.6pt + black)
      if v >= 1 {
        content((x + bar-w / 2, y-of(v) + 0.35), text(size: 14pt, weight: 700)[#fmt2(v)], anchor: "south")
      } else {
        content((x + bar-w / 2, y-of(v) - 0.35), text(size: 14pt, weight: 700)[#fmt2(1 / v)], anchor: "north")
      }
    }
    content((x0 + group-w / 2, bottom - 1.1), text(size: 16pt, weight: 700)[#gname], anchor: "north")
    if captions != none {
      content((x0 + group-w / 2, bottom - 1.7), text(size: 13pt, fill: rgb("#666"))[#captions.at(gi)], anchor: "north")
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
