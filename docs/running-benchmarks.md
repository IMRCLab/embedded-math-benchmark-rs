# Running the Benchmarks

Build and run on the host or a microcontroller. Inputs come from `benchmarks/inputs.json`
(baked in at build time); results print as `BENCH `-prefixed CSV rows, see
[Benchmark I/O Format](io-format.md). For chip names, target triples, and units, see
[Targets & Platforms](platforms.md).

## cargo-make

`cargo install cargo-make` once, then from the repo root:

| Task                     | Result                                                                        |
| ------------------------ | ----------------------------------------------------------------------------- |
| `cargo make bench-host`  | build + run natively                                                          |
| `cargo make bench-stm32`   | build + flash STM32F405 over probe-rs                                       |
| `cargo make bench-rp2040`  | build + flash RP2040 (Pico 1) over probe-rs                                 |
| `cargo make bench-rp2350` | build + flash RP2350 (Pico 2) over probe-rs                                  |
| `cargo make bench-esp32s3` | build + flash ESP32-S3 over its native USB JTAG                            |
| `cargo make bench-stm32-xlto` | STM32F405 with cross-language LTO ([task-categories.md](task-categories.md)) |
| `cargo make bench-rp2350-xlto` | RP2350 with cross-language LTO                                          |
| `cargo make report`      | render `results.csv` into `report.pdf` ([benchmark-viz.md](benchmark-viz.md)) |

Firmware tasks need `probe-rs` and a connected probe, see [HIL Setup](hil-setup.md).
ESP32-S3 additionally needs the `espup`-installed `esp` toolchain on `PATH`.
The `-xlto` tasks additionally need `clang` whose LLVM major matches `rustc -vV`'s, since the two
feed one linker plugin. They are ARM hard-float only.

## Without cargo-make

Build each firmware crate from its own directory (so its crate-local `.cargo/config.toml`
picks the right target), then flash with `probe-rs run --chip <chip>`.
[`Makefile.toml`](../Makefile.toml) has the exact chip name and target triple per platform, e.g.:

```bash
cd benchmarks/mrs-benchmark-stm32
cargo build --release
probe-rs run --chip STM32F405RGTx ../target/thumbv7em-none-eabihf/release/mrs-benchmark-stm32
```

## Collecting results into a CSV

`mrs-benchmark-collect` (`cargo make collect`) greps `BENCH ` rows out of one or more logs
and writes a single headed `results.csv`; it treats each row as opaque text, so the column
format can change without touching the tool. Reads the given files, or stdin when none are
passed; writes to `-o`/`--output` or stdout.

```bash
cargo make bench-host | cargo make collect -- -o results.csv
cargo make collect -- host.log rp2040.log stm32.log -o results.csv   # merge saved logs, as CI does
```

Input with no `BENCH ` rows fails with a non-zero exit.
