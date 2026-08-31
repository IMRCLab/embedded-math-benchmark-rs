# Documentation

| Doc | What it covers |
| --- | -------------- |
| [Running the benchmarks](running-benchmarks.md) | Prerequisites, build, flash and run, per platform |
| [Benchmark I/O format](io-format.md) | `inputs.json` in, `BENCH ` CSV out |
| [Targets & platforms](platforms.md) | Chips, target triples, timing sources, status |
| [Task categories](task-categories.md) | What each task measures, and which comparisons it supports |
| [Build profiles](build-profiles.md) | `release`, `lto`, `size`, `xlto`, and when each is valid |
| [Numerical accuracy](accuracy_evaluation.md) | ULP methodology, and what the accuracy CSV does not support |
| [C reference suites](c-suites.md) | `crazyflie-fw` and `cmsis-dsp`: provenance, naming, wrapper deviations |
| [Benchmark report](benchmark-viz.md) | `results.csv` to `report.pdf`, and the paper figures |
| [Adding a benchmark](adding_a_benchmark.md) | Recipe for a new task |
| [Adding a platform](adding_a_platform.md) | Checklist for a new hardware target |
| [HIL setup](hil-setup.md) | Probe provisioning, CI runners, troubleshooting |

Read [task categories](task-categories.md) before quoting any number as a C-vs-Rust result.

Hardware specified but not yet built: [nRF52840](draft/nrf52840-design.md),
[RP2350 RISC-V](draft/rp2350-riscv-design.md). The original project brief is in
[archive/](archive/project_requirements.md).
