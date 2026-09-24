#import "../theme.typ": accent, accent-note, caption, ink, panel, sec-heading, subheading
#import "@preview/cetz:0.3.4": canvas, draw

#let bar-col = ink

#let lee-rows = (
  ("RP2350 (M33, 150 MHz)", 8.77, "8.8 µs (1,316 cyc)", bar-col),
  ("STM32F405 (M4F, 168 MHz)", 8.95, "9.0 µs (1,503 cyc)", bar-col),
  ("ESP32-S3 (Xtensa, 240 MHz)", 9.58, "9.6 µs (2,299 cyc)", bar-col),
  ("nRF52840 (M4F, 64 MHz)", 24.84, "24.8 µs (1,590 cyc)", bar-col),
)

#let ekf-rows = (
  ("ESP32-S3 (Xtensa, 240 MHz)", 153.68, "154 µs (36,884 cyc)", bar-col),
  ("STM32F405 (M4F, 168 MHz)", 193.44, "193 µs (32,498 cyc)", bar-col),
  ("RP2350 (M33, 150 MHz)", 462.37, "462 µs (69,356 cyc)", bar-col),
  ("nRF52840 (M4F, 64 MHz)", 490.10, "490 µs (31,367 cyc)", bar-col),
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

#let point(title, body) = block(width: 100%, below: 7pt)[
  #text(weight: 800, size: 17.5pt)[#title]
  #v(2pt)
  #text(size: 16.5pt)[#body]
]

#let chip-card(name, arch, detail) = block(
  width: 100%,
  stroke: 0.8pt + rgb("#ccc"),
  inset: (x: 10pt, y: 4.5pt),
  radius: 3pt,
  below: 4.5pt,
)[
  #grid(
    columns: (1fr, auto),
    align: (left + horizon, right + horizon),
    text(weight: 800, size: 16.5pt)[#name],
    text(size: 13.5pt, fill: rgb("#666"))[#arch],
  )
  #v(2pt)
  #text(size: 14pt)[#detail]
]

#let robotics-pipelines-panel() = panel(sec-heading(
  "03",
  "Real-world flight control pipelines across architectures",
  caption: caption[`lto` profile #sym.dot.c median time per step],
))[
  #text(size: 26pt)[
    Two complete control loops built with nalgebra: an SE(3) Lee attitude controller and a
    9-state EKF propagation step, measured on bare metal across five MCU architectures.
  ]
  #v(6pt)

  #grid(
    columns: (1.45fr, 1fr, 1fr),
    column-gutter: 30pt,
    [
      #subheading("Time per step, nalgebra")
      #text(weight: 700, size: 19pt)[Lee controller]
      #v(2pt)
      #pipeline-bars(lee-rows, max-scale: 26, width: 15.5)
      #v(6pt)
      #text(weight: 700, size: 19pt)[EKF step (predict + range update)]
      #v(2pt)
      #pipeline-bars(ekf-rows, max-scale: 520, width: 15.5)
      #v(3pt)
      #caption[All five chips run identical nalgebra algorithms on the same inputs. RP2040 (no FPU) is off the scale: 428 µs for Lee, 6.3 ms for the EKF step.]
    ],
    [
      #subheading("Evaluated MCU architectures")
      #chip-card("STM32F405RG", [M4F #sym.dot.c 168 MHz], [Internal flash with ART accelerator (prefetch buffer and branch cache). Reference flight MCU.])
      #chip-card("ESP32-S3", [Xtensa #sym.dot.c 240 MHz], [Dual-core LX7 with single-precision FPU, external flash with 32 KB instruction cache.])
      #chip-card("RP2350 (Pico 2)", [M33 #sym.dot.c 150 MHz], [ARM Cortex-M33 with FPv5-SP, external QSPI flash with 16 KB XIP cache.])
      #chip-card("nRF52840", [M4F #sym.dot.c 64 MHz], [Cortex-M4F with FPv4-SP, internal flash with conventional wait states.])
      #chip-card("RP2040 (Pico 1)", [M0+ #sym.dot.c 125 MHz], [Dual Cortex-M0+, no hardware FPU (pure software float). Baseline for soft-float penalty.])
    ],
    [
      #subheading("How the chips compare")
      #point("Flash architecture dictates cycles", [
        Identical Cortex-M4F cores diverge by up to 6% in cycle count because STM32's ART accelerator eliminates flash wait states, whereas nRF52840 incurs branch stalls. Wall time scales with the clock (64 vs 168 MHz).
      ])
      #point("Clock frequency overcomes IPC", [
        The ESP32-S3 takes 13% more cycles than the STM32 on the EKF step, but its 240 MHz clock achieves the lowest absolute latency (154 µs).
      ])
      #point("Software float penalty", [
        Without an FPU, the RP2040 (Cortex-M0+) executes floats in software, taking 41 to 49x longer than the fastest chip.
      ])
      #point("RP2350 cache dynamics", [
        Fastest on Lee (1,316 cycles), but incurs 2.1x the cycle count of STM32 on the EKF step (69.4k cycles). See Open Questions below.
      ])
    ],
  )
]
