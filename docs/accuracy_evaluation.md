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
- **1 to 2 ULPs**: Expected floating-point noise from instruction reordering or non-FMA hardware.
- **> 10 ULPs**: Polynomial lookup approximation drift (typical for fast approximations like `micromath`).

### 2. Relative & Absolute Error
- **Relative Error**: $\epsilon_{\text{rel}} = \frac{\|y_{\text{lib}} - y_{\text{ref}}\|_2}{\|y_{\text{ref}}\|_2 + \epsilon}$
- **Absolute Error**: $\epsilon_{\text{abs}} = \|y_{\text{lib}} - y_{\text{ref}}\|_2$

### 3. Geodesic Rotation Angle Error
For quaternion operations (`QuatMul`, `QuatSlerp`, `RotateVector`), the angular error in 3D space is evaluated in radians:
$$\theta_{\text{err}} = 2 \arccos(|q_{\text{lib}} \cdot q_{\text{ref}}|)$$

---

## The composite reference is not neutral

`EkfStep` and `LeeController` have no shared algorithm across libraries, so their f64 reference cannot be neutral. `tasks.py`'s reference follows the Rust operation order. Mean ULP, stm32 `lto`:

| Task | nalgebra | glam | micromath | crazyflie-fw |
| ---- | -------- | ---- | --------- | ------------ |
| `EkfStep` | 0.46 | n/a | 5,624 | 789,913 |
| `LeeController` | 0.15 | 0.16 | 1,305 | 4,183 |

nalgebra's sub-1 ULP is not evidence of accuracy; it means the reference and nalgebra were written from the same reading of the algorithm. `crazyflie-fw`'s number measures **divergence from that algorithm, not error**. Its 5.7% relative error on `EkfStep` is a genuinely different filter rather than bad arithmetic.

The C path calls `kalmanCoreFinalize` three times, uses `kalmanCoreDefaultParams` process noise, carries a 3-DOF attitude *error* state folded into the quaternion at each finalize, and clamps/symmetrizes the covariance. `tasks.py` models none of that. Matching it means porting `kalmanCorePredict`/`ScalarUpdate`/`Finalize` to f64 Python and re-syncing on every submodule bump, and it would only move the asymmetry onto nalgebra.

**Rule:** never quote `crazyflie-fw` composite ULP as accuracy. Compare only within `glam`/`nalgebra`/`micromath`, which do share an algorithm. That comparison is valid, and it shows micromath's fast-math cost in a realistic control step (1,305 vs 0.15 ULP on `LeeController`).

Further cautions on the accuracy CSV:

- ULP is not comparable across tasks. It is scale-relative to each output quantity, so "micromath is 6,030 ULP on `SinCos` but 1,305 on `LeeController`" says nothing about error growth. There is no compounding-drift result in this data.
- `mean_rel_error` is unusable where the reference is near zero: `CrossProduct` reports ~5e5 for every library, including bit-identical ones. Use ULP there.
- Ill-conditioned inputs dominate a ULP mean. `MatInverse3x3` calls a matrix singular at $\kappa(A) \ge 1/\varepsilon_{f32}$ rather than on `|det|`, which moves stm32 `lto` mean ULP from 1,051,177 to 3.25 (nalgebra) and 469,534 to 3.59 (glam).

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
