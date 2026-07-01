# Running the Benchmarks

Build and run on the host or on a microcontroller. Inputs come from
`benchmarks/inputs.json` (compiled in at build time); results print as a CSV block, see
[Benchmark I/O Format](io-format.md). For the list of targets, chip names, and units, see
[Targets & Platforms](platforms.md).

## Host (native)

```bash
cd benchmarks/mrs-benchmark-host
cargo run --release
```

## Microcontroller targets

Build each firmware from its own crate directory (so its crate-local `.cargo/config.toml`
is used), then flash and run with `probe-rs`, which streams RTT output back to your
terminal.

**STM32F405:**

```bash
cd benchmarks/mrs-benchmark-stm32
cargo build --release
probe-rs run --chip STM32F405RGTx ../target/thumbv7em-none-eabihf/release/mrs-benchmark-stm32
```

**Raspberry Pi Pico 1 / 2** follow the same pattern once their crates exist: build, then
`probe-rs run --chip RP2040` (Pico 1) or `--chip RP235x` (Pico 2). The Pico is flashed over
SWD with a separate probe, see [HIL Setup](hil-setup.md) for probe and udev details.

## Shortcuts with cargo-make

`cargo install cargo-make` once, then from the project root:

- **Host:** `cargo make bench-host`
- **STM32:** `cargo make bench-stm`

Per-target shortcuts for the Picos land alongside their crates.
