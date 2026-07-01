# mrs-microbenchmarks

Embedded microbenchmarks for comparing **C** and **Rust** performance and scheduling on
computationally constrained robotic hardware (e.g., Raspberry Pi Pico and STM32).

## Project Overview

This project aims to:

- **Quantify Performance**: Benchmark typical robotic workloads (like controllers, state estimators, and math libraries) implemented in both C and Rust.
- **Evaluate Numeric Solvers**: Compare embedded quadratic programming (QP) solvers (e.g., OSQP, cvxgen vs. generated Rust code) for actuation allocation and MPC.
- **Compare Scheduling**: Analyze real-time scheduling behavior between traditional RTOSs (like FreeRTOS) in C and modern async executors (like Embassy) in Rust.

For the detailed scope and milestones, see the [Full Project Requirements](docs/project_requirements.md) (the original brief; scope evolves).

## Usage

For detailed instructions on building, flashing, and running the benchmarks, see [Running the Benchmarks](docs/running-benchmarks.md)

## Documentation

- [Math Benchmarking Research (Crazyflie / STM32F405)](docs/research/math_benchmarking_crazyflie.md)
- [Targets & Platforms](docs/platforms.md): chips, timing sources, and status
- [Benchmark I/O Format](docs/io-format.md): JSON input and CSV output
- [Hardware-in-the-Loop CI Setup](docs/hil-setup.md): flashing firmware in CI (planned)
