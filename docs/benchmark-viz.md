# Benchmark Report: `viz/plot.py` + CI

Last updated: 2026-08-01

Compares platform x library x task x input on the latest run. CI emits a **PDF report as a pipeline artifact**.

## Fetching artifacts locally

**Stable "latest `main`" link**: https://git.tu-berlin.de/imrc/teaching/multi-robot-systems-project/2026/mrs-microbenchmarks/-/jobs/artifacts/main/raw/report.pdf?job=report-pdf

`glab job artifact <ref> <job>` grabs a job's artifacts from the latest pipeline on that ref
(a branch or tag, not an arbitrary commit):

```bash
glab job artifact $(git branch --show-current) collect-results
glab job artifact $(git branch --show-current) report-pdf
```

## Chart design

- Multipage PDF: Section 1 = one page-set per platform, Section 2 = by-task (below),
  Section 3 = accuracy per platform, Section 4 = Pareto summary table per platform.
  Firmware cycle counts convert to ns via `PLATFORM_CLOCK_HZ` (`viz/config.py`).
- Fixed task grid per page (`TASK_ORDER`, `GRID_SHAPE` in `viz/config.py`); overflow spills
  onto a numbered continuation page. Layout uses `subplots_adjust()` (not `tight_layout()`)
  so a sparse page's grid doesn't inflate.
- Every subplot reserves the platform's full library slot set (`LIBRARY_ORDER`) with a fixed
  library -> color map, so bars align and every page reads as one consistent system.
- Legend entries append each library's version when known (e.g. `glam 0.28.0`,
  `crazyflie-fw @f45ff8f`), read from `library_versions.json` (see
  [docs/io-format.md](io-format.md)). Falls back to the bare name if that file is missing.
- Bar = median per (platform, task, library), overlaid with jittered raw-sample scatter and a
  compact value label (e.g. `1.25k`).
- Subplot title shows the input count (e.g. `MatMul3x3 (n=50)`); an errored library doesn't
  undercount it.
- Y-axis auto-flips to log when cross-library spread exceeds ~10x, with a bold red axis label
  to flag it next to linear neighbors.
- Page header: platform name, `n=` row count, generated-at stamp.

### By-task section

Transposes the grid: one subplot per task, x-axis grouped by platform, bars by library (same
colors/order). No value labels or per-library error breakdown: too many bars per subplot at
up to 5 platforms x 5 libraries. `host` is excluded (not a useful reference next to real
hardware) but still appears last elsewhere.

### Accuracy pages

Section 3 bar-charts mean ULP error per task/library on a symlog axis (`ULP_LINTHRESH`):
linear below 1 ULP so an exact (0-ULP) result gets a real position instead of the old "+1"
hack, log above it. A cosmetic floor (`ZERO_ULP_VISUAL_FLOOR`) keeps 0-ULP bars visible; the
printed label always shows the true value. Section 4 is one Pareto speed/accuracy table per
platform, placed right after that platform's own Section 3 pages, with color-coded
optimal/dominated columns.

Mark/scale/color details live in `viz/pages_time.py` / `viz/pages_accuracy.py`. Read the
code for exact behavior.

## Tool choice: matplotlib + pandas, run via uv

Chosen over plotnine/R-ggplot2/gnuplot for native multipage PDF support (`PdfPages`) and
because explicit code is easier to maintain here than a declarative API for an AI-written,
human-maintained script.

`viz/` is a uv project (`pyproject.toml`/`uv.lock`): `config.py` (constants), `data.py`
(load/transform), `common.py` (shared chart chrome), `pages_time.py` / `pages_accuracy.py`
(page builders), `plot.py` (CLI entry). Same command locally and in CI:
`uv run --project viz viz/plot.py results.csv report.pdf` (`--project` resolves the
environment without changing cwd, so the CSV/PDF paths stay repo-root-relative).

**Verification:** no automated test suite: a plotting script's correctness is fundamentally
visual. Verify by running against a real `results.csv` and reviewing `report.pdf` by eye.

## CI

- `report` stage (between `collect` and `check`), job `report-pdf` in
  [.gitlab/ci/report.yml](../.gitlab/ci/report.yml): `allow_failure: true` (a plotting bug
  shouldn't block a merge) and its own job so it can't fail `collect-results` and can be retried
  alone.
- CI image carries the `uv` binary. `UV_CACHE_DIR`/`UV_PYTHON_INSTALL_DIR` sit on
  `/persist` next to `CARGO_HOME`.
- `cargo make report` runs the same command via `Makefile.toml`.
