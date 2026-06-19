# mrs-microbenchmarks

Embedded microbenchmarks for comparing **C** and **Rust** performance and scheduling on computationally constrained robotic hardware (e.g., STM32 and RP2040).

## Project Overview

This project aims to:
- **Quantify Performance**: Benchmark typical robotic workloads (like controllers, state estimators, and math libraries) implemented in both C and Rust.
- **Evaluate Numeric Solvers**: Compare embedded quadratic programming (QP) solvers (e.g., OSQP, cvxgen vs. generated Rust code) for actuation allocation and MPC.
- **Compare Scheduling**: Analyze real-time scheduling behavior between traditional RTOSs (like FreeRTOS) in C and modern async executors (like Embassy) in Rust.

For the detailed scope, target platforms, and milestones, see the [Full Project Requirements](docs/project_requirements.md).

### Research & Planning
- [Math Benchmarking Research for Crazyflie (STM32F405)](docs/research/math_benchmarking_crazyflie.md)
