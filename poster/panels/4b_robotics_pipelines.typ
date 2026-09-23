#import "../theme.typ": accent, accent-dark, accent-note, caption, panel, sec-heading, subheading
#import "@preview/cetz:0.3.4": canvas, draw

#let stm32-col = accent
#let rp2350-col = rgb("#555")
#let esp-col = black
#let nrf-col = rgb("#aaa")

#let lee-rows = (
  ("RP2350 (M33, 150 MHz)", 8.77, "8.8 µs  (1,316 cyc)", rp2350-col),
  ("STM32F405 (M4F, 168 MHz)", 8.95, "9.0 µs  (1,503 cyc)", stm32-col),
  ("ESP32-S3 (Xtensa, 240 MHz)", 9.58, "9.6 µs  (2,299 cyc)", esp-col),
  ("nRF52840 (M4F, 64 MHz)", 24.84, "24.8 µs (1,590 cyc)", nrf-col),
)

#let ekf-rows = (
  ("ESP32-S3 (Xtensa, 240 MHz)", 153.68, "154 µs (36,884 cyc)", esp-col),
  ("STM32F405 (M4F, 168 MHz)", 193.44, "193 µs (32,498 cyc)", stm32-col),
  ("RP2350 (M33, 150 MHz)", 462.37, "462 µs (69,356 cyc)", rp2350-col),
  ("nRF52840 (M4F, 64 MHz)", 490.10, "490 µs (31,367 cyc)", nrf-col),
)

#let pipeline-bars(rows, max-scale: 500, width: 14.5, bar-h: 0.58, gap: 0.22) = canvas(length: 1cm, {
  import draw: *
  let x-of(v) = v / max-scale * width
  for (i, row) in rows.enumerate() {
    let (label, value, vlabel, color) = row
    let y = -i * (bar-h + gap)
    content((-0.4, y + bar-h / 2), text(size: 15pt)[#label], anchor: "east")
    rect((0, y), (x-of(value), y + bar-h), fill: color, stroke: 0.6pt + black)
    content((x-of(value) + 0.35, y + bar-h / 2), text(size: 15pt, weight: 700)[#vlabel], anchor: "west")
  }
})

#let new-pill = box(fill: accent-dark, inset: (x: 10pt, y: 5pt), radius: 3pt)[
  #text(fill: white, weight: 800, size: 17pt, tracking: 0.03em)[NEW SINCE THE PAPER]
]

#let point(title, body) = block(width: 100%, below: 12pt)[
  #text(weight: 800, size: 19pt)[#title]
  #v(3pt)
  #text(size: 18pt)[#body]
]

#let robotics-pipelines-panel() = panel(sec-heading(
  "03",
  "Beyond primitives: full control steps",
  caption: [#new-pill #h(8pt) #caption[`lto` profile #sym.dot.c median time per step]],
))[
  #text(size: 26pt)[
    Two complete control-loop steps built from the primitives above: a Lee geometric controller and a
    9-state EKF step. Written once in nalgebra and run on four chips.
  ]
  #v(14pt)

  #grid(
    columns: (1.45fr, 1fr, 1fr),
    column-gutter: 30pt,
    [
      #subheading("Time per step, nalgebra")
      #text(weight: 700, size: 19pt)[Lee controller]
      #v(3pt)
      #pipeline-bars(lee-rows, max-scale: 26, width: 15.5)
      #v(12pt)
      #text(weight: 700, size: 19pt)[EKF step (predict + range update)]
      #v(3pt)
      #pipeline-bars(ekf-rows, max-scale: 520, width: 15.5)
      #v(4pt)
      #caption[RP2040 (no FPU) is off the scale: 428 µs for Lee, 6.3 ms for the EKF step.]
    ],
    [
      #subheading("Fast math in a full step")
      #text(size: 19pt)[Swapping nalgebra for micromath on the STM32:]
      #v(10pt)
      #point("Lee controller", [#text(fill: accent, weight: 700)[1.82x] faster, 0.17% mean error (1.8% worst).])
      #point("EKF step", [#text(fill: accent, weight: 700)[1.23x] faster, 0.04% mean error.])
      #v(4pt)
      #accent-note[
        #text(size: 18pt)[
          The speed-for-accuracy trade from section 02 carries over to a full step.
        ]
      ]
      #v(6pt)
      #caption[The Crazyflie C versions are different algorithms, so this section compares chips, not languages.]
    ],
    [
      #subheading("Reading the chips")
      #point("Same core, different clock", [
        nRF52840 and STM32F405 are both Cortex-M4F and land within 6% in cycles. At 64 MHz the nRF takes 2.5 to 2.8x as long.
      ])
      #point("Clock can beat cycles", [
        The ESP32-S3 needs 13% more cycles than the STM32 on the EKF step, but its 240 MHz clock makes it the fastest.
      ])
      #point("RP2350 splits", [
        Fastest on Lee, but it needs 2.1x the STM32's cycles on the EKF step.
      ])
    ],
  )
]
