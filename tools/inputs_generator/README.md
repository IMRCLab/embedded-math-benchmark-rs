# Reproducible Input Generator

This Python subproject is a structured tool to generate mathematically valid, reproducible benchmark input datasets for the microbenchmarks suite.

It is managed via **`uv`**.

## Design

### OOP Structure
All benchmark generators inherit from a `TestCaseGenerator` abstract base class located in `generate_inputs.py`. This ensures a standardized format for defining repetitions, libraries, test names, and generating inputs.

### Edge Cases & Float Precision Limits
To stress-test single-precision floating-point (`f32`) limitations and the behavior of approximation libraries like `micromath` versus exact libraries (`libm`), the generators inject specific float edge cases:
* **Extreme High Coordinates / Scale**: Inputs of magnitude $10^8$ to $10^{15}$ to trigger precision cancellation, loss of significant digits, or overflow boundaries.
* **Subnormals / Underflow**: Coordinates of magnitude $10^{-39}$ or smaller to verify FPU behavior near absolute zero.
* **Trigonometric Period Reductions**: Extreme angles (e.g. $10^{12}$ rad) to test standard modulo-based range reduction against approximate methods.
* **Mathematical Singularities**: Zero determinants and negative square root arguments to verify error handling paths.

## Usage

Ensure you have `uv` installed, then run the generator using `uv run` from this directory:

```bash
# Generate inputs using default seed (42) and write to benchmarks/inputs.json
uv run generate_inputs.py

# Generate using a custom seed for a new dataset
uv run generate_inputs.py --seed 12345

# Specify a custom output path
uv run generate_inputs.py --output /path/to/inputs.json
```
