#import "../theme.typ": accent, stat-block

#let infrastructure-panel() = block(width: 100%)[
  #text(fill: accent, weight: 800, size: 22pt, tracking: 0.05em)[OPEN HARDWARE-IN-THE-LOOP BENCH]
  #v(12pt)
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    column-gutter: 34pt,
    stat-block("18 x 909 math primitives", [x input instances, baked into flash at compile time]),
    stat-block("6 libraries", [glam, nalgebra, micromath, libm vs. CMSIS-DSP and cmath3d]),
    stat-block("5 chips", [STM32F405, RP2040, RP2350, ESP32-S3, nRF52840 (real boards)]),
    stat-block("Cycle-exact", [on-chip counters over RTT, no RTOS jitter]),
  )
]
