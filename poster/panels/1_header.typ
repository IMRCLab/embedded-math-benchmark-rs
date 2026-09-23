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
    [Should Rust firmware wrap existing C math libraries, or use pure-Rust crates?],
    [For embedded math speed, Rust vs C matters least. The math library you link and the build profile matter more.],
    (
      ([10#sym.times], [gap from the math library]),
      ([2.8#sym.times], [gap from the build profile]),
      ([1.47#sym.times], [gap from the language, built fairly]),
    ),
  )
]
