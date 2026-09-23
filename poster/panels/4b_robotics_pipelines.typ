#import "../theme.typ": accent, caption, panel, sec-heading, subheading
#import "@preview/cetz:0.3.4": canvas, draw

#let stm32-col = accent
#let esp-col = black
#let nrf-col = rgb("#888")

#let lee-rows = (
  ("STM32F405 (168 MHz)", 8.95, "9.0 µs  (1,503 cyc)", stm32-col),
  ("ESP32-S3 (240 MHz)", 9.58, "9.6 µs  (2,299 cyc)", esp-col),
  ("nRF52840 (64 MHz)", 24.84, "24.8 µs (1,590 cyc)", nrf-col),
)

#let ekf-rows = (
  ("ESP32-S3 (240 MHz)", 153.68, "153.7 µs (36,884 cyc)", esp-col),
  ("STM32F405 (168 MHz)", 193.45, "193.5 µs (32,499 cyc)", stm32-col),
  ("nRF52840 (64 MHz)", 490.10, "490.1 µs (31,367 cyc)", nrf-col),
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

#let platform-card(name, arch, clock, role, color) = box(
  width: 100%,
  inset: (x: 14pt, y: 11pt),
  stroke: 0.8pt + rgb("#ddd"),
  fill: rgb("#fafafa"),
)[
  #grid(
    columns: (auto, 1fr),
    column-gutter: 10pt,
    align: (left + horizon, left + horizon),
    box(width: 10pt, height: 26pt, fill: color),
    [
      #text(weight: 800, size: 19pt)[#name]
      #h(0.4em)
      #text(fill: rgb("#666"), size: 16pt)[#arch @ #clock]
      #v(2pt)
      #text(size: 15pt, fill: rgb("#555"))[#role]
    ],
  )
]

#let robotics-pipelines-panel() = panel(sec-heading(
  "04",
  "Real-world robotics pipelines across architectures",
  caption: caption[nalgebra · lto profile · median bare-metal execution time],
))[
  #text(size: 26pt)[
    End-to-end SE(3) attitude control and 24-state EKF propagation on bare metal: clock frequency compensates instruction set efficiency.
  ]
  #v(14pt)

  #grid(
    columns: (1.3fr, 1fr, 1.15fr),
    column-gutter: 26pt,
    [
      #subheading("End-to-end latency in microseconds")
      #v(4pt)
      #text(weight: 700, size: 19pt)[Lee Attitude Controller]
      #v(3pt)
      #pipeline-bars(lee-rows, max-scale: 26, width: 14)
      #v(12pt)
      #text(weight: 700, size: 19pt)[EKF 24-State Covariance Propagation]
      #v(3pt)
      #pipeline-bars(ekf-rows, max-scale: 520, width: 14)
      #v(4pt)
      #caption[Evaluated with identical nalgebra code to isolate hardware throughput.]
    ],
    [
      #subheading("Evaluated hardware targets")
      #v(4pt)
      #platform-card("STM32F405", "ARM Cortex-M4F", "168 MHz", "Crazyflie reference flight controller", stm32-col)
      #v(8pt)
      #platform-card("ESP32-S3", "Xtensa LX7", "240 MHz", "Dual-core Wi-Fi & Bluetooth SoC", esp-col)
      #v(8pt)
      #platform-card("nRF52840", "ARM Cortex-M4F", "64 MHz", "Ultra-low-power Bluetooth SoC", nrf-col)
    ],
    [
      #subheading("Key takeaways")
      #v(4pt)
      #text(weight: 800, size: 19pt)[Why identical cores differ]
      #v(3pt)
      #text(size: 17pt)[
        Despite identical Cortex-M4F cores, cycle counts diverge slightly (1,503 vs 1,590 on Lee) due to flash wait states and prefetching: STM32's ART accelerator vs. nRF52's instruction cache.
      ]
      #v(10pt)
      #text(weight: 800, size: 19pt)[Clock frequency vs. IPC]
      #v(3pt)
      #text(size: 17pt)[
        Xtensa needs 17% more cycles on 24-state EKF propagation, but its 240 MHz clock achieves the lowest wall-clock latency (154 µs).
      ]
      #v(10pt)
      #text(weight: 800, size: 19pt)[Sub-10 µs geometric control]
      #v(3pt)
      #text(size: 17pt)[
        Stack allocation and contiguous register passing keep SE(3) attitude control under 10 µs on bare metal with zero heap allocation.
      ]
    ],
  )
]
