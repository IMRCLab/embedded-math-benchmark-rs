# Lee Controller Runtime Evaluation

Absolute execution time comparison of the Lee Controller across host and embedded platforms running at nominal frequencies:
*   **Host (native)**
*   **STM32F405 (Crazyflie MCU)**: 168 MHz
*   **RP2350-arm (Raspberry Pi Pico 2)**: 150 MHz
*   **RP2040 (Raspberry Pi Pico 1)**: 125 MHz

## Converted Absolute Execution Time

Median execution time per iteration in microseconds ($\mu\text{s}$):

| Platform | Library | Active Clock Frequency | Median Time ($\mu\text{s}$) |
| :--- | :--- | :--- | :--- |
| **host** | glam | *N/A* | **0.0815** |
| **host** | micromath | *N/A* | **0.0835** |
| **host** | nalgebra | *N/A* | **0.0839** |
| **rp2040** | glam | 125 MHz | **455.28** |
| **rp2040** | micromath | 125 MHz | **456.17** |
| **rp2040** | nalgebra | 125 MHz | **461.42** |
| **rp2350-arm** | glam | 150 MHz | **15.80** |
| **rp2350-arm** | micromath | 150 MHz | **5.38** |
| **rp2350-arm** | nalgebra | 150 MHz | **9.71** |
| **stm32** | glam | 168 MHz | **10.72** |
| **stm32** | micromath | 168 MHz | **5.79** |
| **stm32** | nalgebra | 168 MHz | **11.07** |

---

![Lee Controller Runtime Comparison](lee_controller_runtime.png)
