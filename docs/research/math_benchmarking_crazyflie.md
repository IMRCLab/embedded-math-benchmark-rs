# Benchmarking no_std Rust Math Libraries on STM32F405 (Crazyflie)

## Target Hardware Constraints
* **MCU**: STM32F405 (Cortex-M4F core, 168 MHz).
* **FPU**: Single-precision (f32) hardware floating-point unit. Benchmarks must strictly use `f32` to avoid double-precision (`f64`) software emulation.
* **Target Triple**: `thumbv7em-none-eabihf` to leverage hardware FPU instructions.
* **Measurement**: DWT (Data Watchpoint and Trace) `CYCCNT` register for cycle-accurate benchmarking.
* **Compiler Defeat**: Inputs/outputs wrapped in `core::hint::black_box()` to prevent compile-time optimizations.
* **Library Candidates**: `libm` (IEEE 754 compliant) vs. `micromath` (fast, approximate).

## Libraries to Explore and Test
For robotics controllers and state estimators, we need linear algebra and 3D geometry math libraries that run in `no_std` environments. The primary candidates to benchmark are:
* **nalgebra**: Standard Rust linear algebra library. Features comprehensive matrix operations, transforms, and geometry types. Fully compatible with `no_std`.
* **glam**: Highly optimized 3D vector math library. Uses SIMD where available and is popular for fast 3D transformations. Fully compatible with `no_std`.
* **libm**: Provides standard math functions (trigonometric, exponential, log) for `no_std` targets. Note that `libm` serves as the underlying backend for both `nalgebra` and `glam` in `no_std` configurations.
* **micromath**: Fast, approximation-based alternatives for transcendental math functions, designed specifically for low-latency embedded use.
  * *Vector/Matrix Support*: `micromath` provides basic 2D/3D vectors and quaternions out of the box, but lacks matrix types or general linear algebra.
  * *Optional Exploration*: We could extend `nalgebra` to support `micromath` by implementing `RealField` for a custom `f32` wrapper (e.g. `MicroF32`), or write a custom stack-allocated vector/matrix library on top of `micromath`'s transcendental functions.

## Architectural Paths for Benchmarking

### Option 1: Custom Rust Application ("Bare Metal")
Bypass the existing drone firmware and run a standalone `no_std` binary using the `cortex-m-rt` crate.

* **Pros**:
  * **Zero OS Interference**: No RTOS scheduler or context switches affecting cycle measurements.
  * **Modern Tooling**: Direct integration with standard embedded Rust tools (e.g., `probe-rs`, `defmt`).
* **Cons**:
  * **Hardware Modification**: Requires connecting a debug probe (e.g., ST-Link) to the SWD pads on the Crazyflie.
  * **Clock Configuration**: Manual clock configuration is required via HAL to ensure the system runs at 168 MHz rather than the default 16 MHz internal oscillator.

### Option 2: FreeRTOS Test Harness (Crazyflie Firmware Integration)
Compile Rust benchmarks as a static library (`crate-type = ["staticlib"]`) with an `extern "C"` interface, linking it into the Crazyflie CMake build.

* **Pros**:
  * **Over-the-Air Benchmarking**: Flashing via Crazyradio and streaming results using the firmware log system.
  * **Environment Fidelity**: Hardware clocks and peripheral states are pre-configured to production flight settings.
* **Cons**:
  * **Preemption Jitter**: FreeRTOS preemption (SysTick, radio, stabilizer tasks) can interrupt measurements.
  * **Mitigation Required**: The measurement block must be wrapped in a critical section (e.g., `cortex_m::interrupt::free` or `taskENTER_CRITICAL()`) to disable interrupts.

## Open Question: Do We Need Accuracy Benchmarking?
Since libraries like `micromath` use approximations to trade accuracy for execution speed, we should decide whether to benchmark precision alongside timing performance.
