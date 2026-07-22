# Benchmark Framework Documentation

Welcome to the documentation for the `mrs-microbenchmarks` framework! 

This framework is built around a centralized, JSON-driven architecture that allows you to cleanly separate hardware platforms, test definitions, and library implementations without code duplication.

Use this guide as an index to explore the different components of the framework.

## General Usage

- **[Running the Benchmarks](running-benchmarks.md)**  
  Instructions on how to compile, flash, and execute the benchmarks on the host machine as well as microcontrollers (STM32, RP2040) using `probe-rs`.

## Reports

- **[Benchmark Report](benchmark-viz.md)**  
  How `results.csv` becomes the per-platform `report.pdf`: chart design, the `report` CI stage, and the stable "latest `main`" link.

## Development & Configuration

- **[Inputs Configuration (`inputs.json`)](inputs_configuration.md)**  
  Learn how to define test cases, repetitions, target platforms, and target libraries using the central JSON configuration file.
  
- **[Adding a New Benchmark](adding_a_benchmark.md)**  
  A step-by-step guide for developers on how to write new microbenchmark tasks (defining the inputs, registering the task, and writing the math operations) and integrate them across platforms.

- **[Benchmark I/O Format](io-format.md)**  
  Documentation detailing exactly how benchmark data is logged out of the platforms via standard serial output in CSV format, perfect for offline Python evaluation.

## Hardware & Environment

- **[Targets & Platforms](platforms.md)**  
  Details the supported hardware chips, timing sources (e.g. `DWT` cycle counters, SysTick), and the current integration status.

- **[Hardware-in-the-Loop (HIL) Setup](hil-setup.md)**  
  Setup instructions and udev rules required for configuring the hardware probes for automated CI flashing.

## Project Background

- **[Full Project Requirements](project_requirements.md)**  
  The original scope, goals, and milestones for the robotic benchmarking project.

- **[Math Benchmarking Research](research/math_benchmarking_crazyflie.md)**  
  Initial background research on how to evaluate math for the Crazyflie firmware.

- **[Crazyflie C Math Integration](crazyflie_math_integration.md)**  
  Detailed architecture of the `crazyflie-fw` FFI reference suite, including required installs and cross-compilation workarounds.

## Evaluation & Results

- **[Lee Controller Evaluation](lee_controller_evaluation.md)**  
  Runtime evaluation results and execution time comparison of the Lee Controller across host and embedded platforms.
