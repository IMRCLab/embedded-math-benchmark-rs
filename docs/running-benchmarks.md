# Running the Benchmarks

This document provides instructions on how to build and execute the benchmarks on both the host system and the target microcontroller.

## STM32 Target (STM32F405RGTx)

To build and run the STM32 benchmark on the physical board over an ST-Link/V2 debugger:

1. **Build the firmware**:
   ```bash
   cd benchmarks/mrs-benchmark-stm32
   cargo build --release
   ```

2. **Flash and run**:
   ```bash
   probe-rs run --chip STM32F405RGTx ../target/thumbv7em-none-eabihf/release/mrs-benchmark-stm32
   ```

## Host Target (Native)

To build and run the host-side benchmark natively on your local machine:
```bash
cd benchmarks/mrs-benchmark-host
cargo run --release
```

## Shortcuts with cargo-make

If you have `cargo-make` installed (`cargo install cargo-make`), you can build and run the benchmarks directly from the project root:

- **Host (Native)**:
  ```bash
  cargo make bench-host
  ```
- **STM32 Target**:
  ```bash
  cargo make bench-stm
  ```

