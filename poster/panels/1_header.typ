#import "../theme.typ": accent, hero-banner, qr-code

#let header-panel() = [
  #grid(
    columns: (1fr, auto),
    column-gutter: 30pt,
    align: (left, right),
    [
      #text(fill: accent, weight: 800, size: 22pt, tracking: 0.08em)[
        #upper("IROS 2026 / workshop on Rust for robotics")
      ]
      #v(8pt)
      #text(weight: 800, size: 74pt)[Benchmarking Rust and C Math Libraries for Embedded Robotics]
      #v(14pt)
      #text(size: 28pt)[
        Pascal Harprecht#text(fill: accent)[\*] #h(0.5em) #sym.dot.c #h(0.5em) Thomas Lübbehüsen#text(fill: accent)[\*] #h(0.5em) #sym.dot.c #h(0.5em) Wolfgang Hönig
        #h(0.8em) #sym.dot.c #h(0.8em) Technical University of Berlin #h(0.5em) #sym.dot.c #h(0.5em) #text(fill: accent)[\*]equal contribution
      ]
    ],
    [
      #qr-code("https://github.com/IMRCLab/embedded-math-benchmark-rs", size: 9cm)
      #v(8pt)
      #align(center)[#text(fill: accent, weight: 800, size: 18pt)[#upper("paper, code & full tables")]]
    ],
  )
  #v(20pt)

  #hero-banner(
    [Across 18 primitives, 6 libraries and 4 build profiles, the language was the weakest predictor in the grid.],
    [1.5#sym.times],
    [Matched on ABI class and build profile, C and Rust land within 1.5x of each other in both directions.],
  )
]
