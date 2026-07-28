# Benchmark Report: `viz/plot.py` + CI

Last updated: 2026-07-23

**Stable "latest `main`" link** (GitLab's latest-successful-artifact URL):

https://git.tu-berlin.de/imrc/teaching/multi-robot-systems-project/2026/mrs-microbenchmarks/-/jobs/artifacts/main/raw/report.pdf?job=report-pdf

Compares platform x library x task x input on the latest run. CI emits a **PDF report as a
pipeline artifact**.

## Chart design

- Multipage PDF: first a page per platform, then a final section of pages grouped by task
  instead (see "By-task section" below). All times are converted to nanoseconds before
  plotting: firmware platforms report raw core cycles and are
  divided by that platform's clock rate (`PLATFORM_CLOCK_HZ` in `viz/plot.py`, hardcoded from
  each `mrs-benchmark-<platform>/src/main.rs`'s clock config)
- Fixed grid of task subplots per page, in a fixed order (`TASK_ORDER` in `viz/plot.py`,
  mirroring `inputs.json`'s case order). A task beyond one page's capacity spills onto a
  numbered continuation page. Subplot geometry is fixed via `fig.subplots_adjust()` (not
  `tight_layout()`), so every page's grid is identical regardless of how many cells it uses —
  a sparsely populated continuation page doesn't inflate its subplots.
- Every subplot on a platform's page reserves the same fixed set of library slots (that
  platform's libraries, in `LIBRARY_ORDER`), so every subplot has identical width and the page reads as one consistent set.
- Each subplot title includes the input count, e.g. `MatMul3x3 (n=50)` — the max sample count
  across that subplot's library slots, so an errored-out library (already called out by the
  grey error footnote) doesn't undercount it.
- Each subplot: bar = median per (platform, task, library), overlaid with a jittered scatter of
  every raw per-iteration sample, plus a compact value label (e.g. `1.25k`) above the bar,
  cleared of the jitter cloud.
- Y-scale auto-flips to log when cross-library spread exceeds ~10x (only the transcendental
  tasks hit this on a per-platform page, but it's the common case on by-task pages since host
  is typically orders of magnitude faster than firmware); the y-axis label turns bold red on a
  log subplot so it can't be missed next to a linear neighbor.
- Fixed library -> color map; crazyflie-fw (the only C impl) gets a distinct hue
- Page header: platform name (bold), `n=` row count (italic subtitle), generated-at stamp
  top-right.

### By-task section

Appended after the per-platform pages: the same grid/pagination mechanic, transposed. One
subplot per task; x-axis grouped by platform (`PLATFORM_ORDER`), bars within each group by
library (same colors/order as the per-platform pages). Every platform group reserves the full
library set so all groups are the same width, mirroring the per-platform pages' own
same-width invariant one level down. Exists to compare whether libraries perform consistently
relative to each other across platforms. Value labels are rotated 90° here (not on the
per-platform pages) since up to 5 platforms x 5 libraries per subplot makes horizontal labels
on similarly-tall neighboring bars run together; the error footnote is a single total count
rather than a per-library breakdown for the same reason.

Mark/scale/color logic lives in `viz/plot.py` itself, read the code for exact behavior.

## Tool choice: matplotlib + pandas, run via uv

Over plotnine/R-ggplot2/gnuplot: native multipage PDF (`PdfPages`), matures into the later
paper-figure pass, and explicit code beats declarative for an AI-written, human-maintained
script (plotnine's brevity win vanishes; its hidden dodge/free-scale debugging cost stays).

Deps via **uv**: PEP 723 inline block plus a committed `viz/plot.py.lock`. Same command locally
and in CI: `uv run viz/plot.py results.csv report.pdf`. uv builds a pinned ephemeral venv, so
both run identical versions.

**Verification:** no automated test suite, a plotting script's correctness is fundamentally
visual. Verify by running against a real `results.csv` and reviewing `report.pdf` by eye.

## CI

- `report` stage (between `collect` and `check`), job `report-pdf` in
  [.gitlab/ci/report.yml](../.gitlab/ci/report.yml): `allow_failure: true` (a plotting bug
  shouldn't block a merge) and its own job so it can't fail `collect-results` and can be retried
  alone.
- CI image carries the `uv` binary. `UV_CACHE_DIR`/`UV_PYTHON_INSTALL_DIR` sit on
  `/persist` next to `CARGO_HOME`.
- `cargo make report` runs the same command via `Makefile.toml`.
