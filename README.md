# mrs-microbenchmarks

Embedded microbenchmarks for comparing **C** and **Rust** performance and scheduling on
computationally constrained robotic hardware (e.g., Raspberry Pi Pico and STM32).

## Project Overview

This project aims to:

- **Quantify Performance**: Benchmark typical robotic workloads (like controllers, state estimators, and math libraries) implemented in both C and Rust.
- **Evaluate Numeric Solvers**: Compare embedded quadratic programming (QP) solvers (e.g., OSQP, cvxgen vs. generated Rust code) for actuation allocation and MPC.
- **Compare Scheduling**: Analyze real-time scheduling behavior between traditional RTOSs (like FreeRTOS) in C and modern async executors (like Embassy) in Rust.

Milestone 1, the math-library comparison, is done and running on hardware in CI across five
platforms and four build profiles. QP solvers and scheduling are later milestones. The results
are written up in `rust_for_robotics_workshop_iros_2026.pdf` at the repo root; the original
brief is in [docs/archive/](docs/archive/project_requirements.md).

## Usage

For detailed instructions on building, flashing, and running the benchmarks, see [Running the Benchmarks](docs/running-benchmarks.md)

## Documentation

All guides, research, format specifications, and hardware setup instructions can be found in the **[Benchmark Framework Documentation](docs/benchmark.md)** overview.
