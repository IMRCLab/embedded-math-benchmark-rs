# Benchmark Plotting Script: Design

Last updated: 2026-07-16

Design for `viz/plot.py`. Infrastructure (CI `report` stage, PDF artifact, stable link, image
deps) lives in [benchmark-viz-ci-design.md](benchmark-viz-ci-design.md). This doc is the plots
and the script only.

## Data facts that drive the design

From a real `results.csv` (4162 rows: 4 platforms × 5 libraries × 8 tasks × 50 inputs, ragged).

- **`per = duration / repetitions`** is already the mean over the reps loop for one input, so
  the 50 values per (platform, library, task) are an **input-sensitivity envelope**, not
  measurement noise.
- **The input axis is nearly flat for most groups:** median coefficient of variation across
  the 50 inputs is ~3%, often 0 (constant cycle count). So `x = input_index` would draw ~250
  near-identical bars per subplot. **Inputs get collapsed.**
- **A few groups are strongly input-sensitive, and that's a result:** libm and crazyflie-fw
  transcendentals (SinCos, Atan2) have data-dependent cost (SinCos spans 16× across inputs on
  host, 5-7× on MCUs); micromath's polynomial versions are constant-time. Whiskers must show it.
- **Cross-library spread within a task is mostly modest (median 1.9×) but 15-29× for the
  transcendentals** (SinCos: micromath 149 vs cf-fw 3981 cyc on stm32). Y-scale must adapt.
- **Coverage is ragged:** libm does scalar ops only (Atan2/SinCos/Sqrt); MatInverse3x3 is
  glam+nalgebra only.
- **93 `ERROR:` rows** (e.g. singular MatInverse3x3), dropped and counted.
- **C vs Rust:** crazyflie-fw is the only C impl (FFI); glam, nalgebra, micromath, libm are
  Rust (libm = the Rust crate).

## Tool: matplotlib + pandas, run via uv

Over plotnine, R/ggplot2, `plotters`, gnuplot, JS. Reasons that survive "AI writes the code":

- **Native multipage PDF** (`PdfPages`), one page per platform. plotnine has none and would
  pull matplotlib back in to stitch.
- **Standard paper-figure tool**, so the script matures into the paper pass.
- **Explicit beats declarative** for AI-written, human-maintained code: plotnine's brevity win
  vanishes, its hidden dodge/free-scale debugging cost stays.

Deps via **uv**: PEP 723 inline block (`matplotlib`, `pandas`) plus a committed `uv.lock`. Run
`uv run viz/plot.py results.csv report.pdf` locally and in CI; uv builds a pinned ephemeral
venv, so both run identical versions (better than apt, which drifts by Debian release). The CI
image carries the `uv` binary (see [benchmark-viz-ci-design.md](benchmark-viz-ci-design.md) §3).

## Chart design

**One mark per (platform, task, library):** median plus min/max whiskers. Min/max, not p5/p95:
the slow-input tail is a result, and worst-case latency matters for real-time robotics.

**Layout**

- Multipage PDF, one page per platform (host `ns`; stm32/rp2040/rp2350-arm `cycles`). One unit
  per page, so no axis mixes units and no cycle↔ns conversion is needed.
- Fixed **4×2 grid of task subplots** per page; blank cells for tasks a platform skipped.
- Each subplot: x = library, y = per-iter time, own y-axis.

**Per-subplot y-scale:** auto **log if cross-library max/min > ~10× else linear**, noted in the
subplot title. Only the 2-3 transcendental subplots flip to log.

**Mark follows scale** (bars mis-encode on log):

- linear → **bar + whisker**
- log → **dot + whisker**

**Color & order (the C-vs-Rust thesis)**

- One fixed library→color map, reused in every subplot and page.
- Rust libs get a colorblind-safe set; **crazyflie-fw (C) gets a distinct hue plus a hatch**,
  always rightmost, so the C mark is findable at a glance.
- Swatches come from the `dataviz` skill at implementation time.

**Annotations:** page title `platform · unit · n rows (dropped K errors)`; per-subplot note
when a library had errored inputs.

## Script structure

Single file (~250 lines), PEP 723 header, `uv.lock` alongside.

```
load_and_clean(csv)                -> df_ok, error_counts   # add `per`, split ERROR rows
aggregate(df_ok)                   -> per-(platform,task,library) median/min/max
plot_platform_page(agg, platform)  -> Figure                # 4x2 grid, scale + mark logic
main(csv, out_pdf)                 -> loop platforms into PdfPages
```

## Deferred

- **Per-input strip** behind the dot on log subplots, to show where the mass sits.
- **Cross-platform on one page:** needs cycle↔ns conversion; out of scope per the infra doc.
