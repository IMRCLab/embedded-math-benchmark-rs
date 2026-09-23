# Numerical accuracy

How the microbenchmark outputs are compared against a 64-bit reference, and what that comparison
does and does not support. Task definitions, input generators, f64 reference calculators and
accuracy evaluators all live in `tools/inputs_generator/tasks.py`; adding a task there is covered
in [adding_a_benchmark.md](adding_a_benchmark.md).

## Metrics

**ULP distance** is the bit-level distance between the target's f32 output and the f64 reference
rounded to f32:

$$\text{ULP}(y_{\text{lib}}, y_{\text{ref}}) = \frac{|y_{\text{lib}} - y_{\text{ref}}|}{2^{E(y_{\text{ref}}) - 23}}$$

0 ULP is a bit-exact match. 1 to 2 ULP is ordinary floating-point noise from instruction
reordering or a missing FMA. Anything above ~10 ULP is polynomial-approximation drift, which in
this suite means micromath.

Alongside it the evaluator reports relative error
$\|y_{\text{lib}} - y_{\text{ref}}\|_2 / (\|y_{\text{ref}}\|_2 + \epsilon)$, absolute error
$\|y_{\text{lib}} - y_{\text{ref}}\|_2$, and for quaternion tasks the geodesic rotation angle
$\theta_{\text{err}} = 2\arccos(|q_{\text{lib}} \cdot q_{\text{ref}}|)$ in radians.

`cargo make eval-accuracy` writes `accuracy_results.csv` and `accuracy_results.json` plus a
terminal summary table.

## What the numbers look like

Current values live in `accuracy_results.csv` and section 3 of `report.pdf`; treat anything quoted
here as an order of magnitude, not a figure to cite.

On pure linear algebra every library stays within ~15 mean ULP of the reference, on inputs that are
not singular in f32. The gap that matters is fast-math approximation: at stm32 `lto`, micromath is
around 1.2e5 mean ULP on `Sqrt` and 5.4e5 on `Atan2`, against 0.07 and 0.14 for both `libm` and
newlib. That is three to six orders of magnitude, and it is not uniform across functions, so read
the per-task rows rather than assuming one figure for the crate.

`newlib` in the accuracy output is the `crazyflie-fw` column for the five single-transcendental
tasks, where no firmware code runs, see [c-suites.md](c-suites.md#naming).

## The composite reference is not neutral

`EkfStep` and `LeeController` have no shared algorithm across libraries, so their f64 reference cannot be neutral. `tasks.py`'s reference follows the Rust operation order. Mean ULP, stm32 `lto`:

| Task | nalgebra | glam | micromath | crazyflie-fw |
| ---- | -------- | ---- | --------- | ------------ |
| `EkfStep` | 0.46 | n/a | 5,624 | 814,137 |
| `LeeController` | 0.15 | 0.16 | 1,305 | 4,183 |

nalgebra's sub-1 ULP is not evidence of accuracy; it means the reference and nalgebra were written from the same reading of the algorithm. `crazyflie-fw`'s number measures **divergence from that algorithm, not error**. Its 5.5% relative error on `EkfStep` is a genuinely different filter rather than bad arithmetic.

The C path calls `kalmanCoreFinalize` three times, uses `kalmanCoreDefaultParams` process noise, carries a 3-DOF attitude *error* state folded into the quaternion at each finalize, and clamps/symmetrizes the covariance. `tasks.py` models none of that. Matching it means porting `kalmanCorePredict`/`ScalarUpdate`/`Finalize` to f64 Python and re-syncing on every submodule bump, and it would only move the asymmetry onto nalgebra.

**Rule:** never quote `crazyflie-fw` composite ULP as accuracy. Compare only within `glam`/`nalgebra`/`micromath`, which do share an algorithm. That comparison is valid, and it shows micromath's fast-math cost in a realistic control step (1,305 vs 0.15 ULP on `LeeController`).

Further cautions on the accuracy CSV:

- ULP is not comparable across tasks. It is scale-relative to each output quantity, so "micromath is 6,030 ULP on `SinCos` but 1,305 on `LeeController`" says nothing about error growth. There is no compounding-drift result in this data.
- `mean_rel_error` is unusable where the reference is near zero: `CrossProduct` reports ~5e5 for every library, including bit-identical ones. Use ULP there.
- Ill-conditioned inputs dominate a ULP mean. `MatInverse3x3` calls a matrix singular at $\kappa(A) \ge 1/\varepsilon_{f32}$ rather than on `|det|`, which moves stm32 `lto` mean ULP from 1,051,177 to 3.25 (nalgebra) and 469,534 to 3.59 (glam).
