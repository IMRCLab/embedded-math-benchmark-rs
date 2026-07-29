# Numerical Accuracy & Evaluation Framework

This document describes the design, scientific metrics, and workflows used to evaluate the numerical accuracy of embedded math libraries (`glam`, `nalgebra`, `micromath`, `crazyflie-fw`, `libm`) against high-precision 64-bit floating-point (`f64`) reference ground truth.

## Architecture: Single Point of Reference (`tasks.py`)

All task definitions, supported library mappings, input generators, 64-bit reference calculators, and accuracy evaluators are co-located in a single file: `tools/inputs_generator/tasks.py`.

```
tools/inputs_generator/
├── pyproject.toml              # Python & dependencies configuration (managed via uv)
├── tasks.py                    # Single source of truth for all benchmark tasks
├── generate_inputs.py          # Generates benchmarks/inputs.json
├── generate_reference.py       # Generates benchmarks/reference_results.json (f64 reference)
└── evaluate_accuracy.py        # Compares results.csv against reference_results.json
```

### Adding a New Task
To add a new benchmark task or evaluation rule, subclass `BenchmarkTask` in `tools/inputs_generator/tasks.py` and register it in `TASK_REGISTRY`. The input generator, reference calculator, and accuracy evaluator automatically pick it up without modifying any CLI scripts.

---

## Accuracy Metrics & Scientific Methodology

### 1. ULP Distance (Units in the Last Place)
ULP distance measures the bit-level distance between single-precision IEEE 754 floats:
$$\text{ULP}(y_{\text{lib}}, y_{\text{ref}}) = \frac{|y_{\text{lib}} - y_{\text{ref}}|}{2^{E(y_{\text{ref}}) - 23}}$$

- **0 ULP**: Bit-exact match with the nearest valid single-precision float representation.
- **1–2 ULPs**: Expected floating-point noise from instruction reordering or non-FMA hardware.
- **> 10 ULPs**: Polynomial lookup approximation drift (typical for fast approximations like `micromath`).

### 2. Relative & Absolute Error
- **Relative Error**: $\epsilon_{\text{rel}} = \frac{\|y_{\text{lib}} - y_{\text{ref}}\|_2}{\|y_{\text{ref}}\|_2 + \epsilon}$
- **Absolute Error**: $\epsilon_{\text{abs}} = \|y_{\text{lib}} - y_{\text{ref}}\|_2$

### 3. Geodesic Rotation Angle Error
For quaternion operations (`QuatMul`, `QuatSlerp`, `RotateVector`), the angular error in 3D space is evaluated in radians:
$$\theta_{\text{err}} = 2 \arccos(|q_{\text{lib}} \cdot q_{\text{ref}}|)$$

---

## Command Workflows (`cargo-make`)

All workflows are automated via `cargo-make` tasks:

```bash
# 1. Generate inputs.json from task definitions
cargo make inputs

# 2. Compute 64-bit reference ground truth
cargo make generate-ref

# 3. Run benchmarks and collect results into results.csv
cargo make bench-host | cargo make collect -- -o results.csv

# 4. Evaluate numerical accuracy of results.csv against reference ground truth
cargo make eval-accuracy
```

Outputs:
- Terminal Markdown summary table comparing Max ULP, Mean ULP, Relative Error, and % Bit-Exact runs.
- `accuracy_results.csv` and `accuracy_results.json` containing structured metrics per platform, library, and task.
