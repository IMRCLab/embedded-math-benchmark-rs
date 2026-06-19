# Embedded Microbenchmarks for Computationally Constrained Robots

Multi-Robot Systems show advantages in robustness and resilience through teamwork. It is
common to use simpler and cheaper robots to achieve complicated tasks by collaboration of a
potentially heterogeneous robot team. Often such robots are energy- and compute-constrained
and rely on microcontrollers (e.g., RP2040 on Pololu robots, STM32 on Crazyflie robots)
to function properly. In the past, such robots were programmed in C, an efficient compiled
programming language that unfortunately brings safety and security risks. A more modern
approach is using Rust, a compiled memory-safe language, that has good embedded support
with a reduced “core” feature set.

The aim of this project is to rigorously compare the two programming languages and libraries
with respect to their usefulness for multi-robot systems. Specifically, the outcome of this project
is to a) quantify the performance difference of typical robotic algorithms (e.g., state estimation,
control) between languages and chips, b) identify which math libraries are particularly well
suited, and c) provide an insight into the scheduling behavior of robotics-relevant frameworks
(FreeRTOS, embassy). This should be achieved through microbenchmarks that run directly on
the existing robotic hardware.

## Work Outline / Milestones

Depending on the size of the group, the scope may vary and can be defined at the beginning of
the project. Concrete ideas include:
1. Benchmark embedded math libraries on STM32, RP2040 for Rust vs. C for workloads
such as robot controllers and robot state estimators.
2. Benchmark embedded numeric solvers (e.g., QP-solvers) for typical problem instances
(e.g., actuation allocation, MPC). To this end, it would also be in the scope of the
project to automatically generate Rust-code to solve fixed optimization problems. Similar
libraries in C are OSQP or cvxgen.
3. For robotics the use of real-time operating systems (RTOS) is often helpful to guarantee
proper timing of the various control and state estimation loops. In C, FreeRTOS or nuttX
are common RTOS. For Rust, the dominant framework is embassy which promotes an
async model, although others exist, see here and here. One open question is if embassy
can provide consistent scheduling, similar to FreeRTOS.
