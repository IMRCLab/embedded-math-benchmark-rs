# embedded-math-benchmark-rs

Microbenchmarks comparing C and Rust math libraries on the compute-constrained
microcontrollers used in multi-robot systems (STM32, RP2040, RP2350, ESP32-S3, nRF52840).
One JSON config defines every task; proc-macros bake it into `no_std` firmware that runs on
each platform and reports both cycle counts and numerical accuracy.

**[Latest benchmark report (PDF)](https://github.com/IMRCLab/embedded-math-benchmark-rs/releases/latest/download/report.pdf)**
([raw results.csv](https://github.com/IMRCLab/embedded-math-benchmark-rs/releases/latest/download/results.csv)).

## Documentation

| Doc                                                  | What it covers                                                         |
| ---------------------------------------------------- | ---------------------------------------------------------------------- |
| [Running the benchmarks](docs/running-benchmarks.md) | Prerequisites, build, flash and run, per platform                      |
| [Benchmark I/O format](docs/io-format.md)            | `inputs.json` in, `BENCH ` CSV out                                     |
| [Targets and platforms](docs/platforms.md)           | Chips, target triples, timing sources, status                          |
| [Task categories](docs/task-categories.md)           | What each task measures, and which comparisons it supports             |
| [Build profiles](docs/build-profiles.md)             | `release`, `lto`, `size`, `xlto`, and when each is valid               |
| [Numerical accuracy](docs/accuracy_evaluation.md)    | ULP methodology, and what the accuracy CSV does not support            |
| [C reference suites](docs/c-suites.md)               | `crazyflie-fw` and `cmsis-dsp`: provenance, naming, wrapper deviations |
| [Benchmark report](docs/benchmark-viz.md)            | `results.csv` to `report.pdf`, and the paper figures                   |
| [Adding a benchmark](docs/adding_a_benchmark.md)     | Recipe for a new task                                                  |
| [Adding a platform](docs/adding_a_platform.md)       | Checklist for a new hardware target                                    |
| [HIL setup](docs/hil-setup.md)                       | Probe provisioning, CI runners, troubleshooting                        |

## Quick start

```bash
cargo install cargo-make
cargo make bench-host | cargo make collect -- -o results.csv
cargo make report
```

Actual firmware runs need `probe-rs` and a connected probe, see
[running the benchmarks](docs/running-benchmarks.md). Results land in `results.csv` and
`accuracy_results.csv` and render to `report.pdf`.

## Layout

- `benchmarks/`: Cargo workspace. `mrs-benchmark-core` holds the traits, task registry and
  per-library suites; `-macros` bakes `inputs.json` in at build time; `-host` and one
  `no_std` crate per platform run it; `-collect` merges logs into `results.csv`
- `docs/`
- `viz/`: renders `results.csv` into `report.pdf`
- `tools/`: standalone scripts (input generator, RP2350 rescue)
