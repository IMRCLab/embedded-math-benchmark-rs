#import "../theme.typ": accent, stat-block, subheading

#let infrastructure-panel() = block(width: 100%)[
  #subheading("open hardware-in-the-loop bench", color: accent)
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr),
    column-gutter: 34pt,
    stat-block("18 math primitives", [Tested against various inputs. The tests are baked into flash at compile time]),
    stat-block(
      "6 libraries",
      [glam, nalgebra, micromath, Rust libm vs. CMSIS-DSP and cmath3d (Crazyflie firmware's own 3D math library)],
    ),
    stat-block("5 chips", [STM32F405, RP2040 (Pico 1), RP2350 (Pico 2), ESP32-S3, nRF52840 ]),
    stat-block("Cycle-exact", [on-chip counters, logged over RTT]),
  )
]
