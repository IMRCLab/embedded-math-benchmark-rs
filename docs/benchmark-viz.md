# Benchmark Report: `viz/plot.py` + CI

Last updated: 2026-07-22 (task order, log-scale highlight, fixed-width bars, pagination, ns conversion)

Compares platform x library x task x input on the latest run. CI emits a **PDF report as a
pipeline artifact**: no dashboard, hosting, or service.

```
run-host / run-mcu -> collect-results -> report-pdf
                      (results.csv)       (viz/plot.py -> report.pdf)
```

## Chart design

- Multipage PDF, one page per platform. All times are converted to nanoseconds before
  plotting: host reports `ns` natively, firmware platforms report raw core cycles and are
  divided by that platform's clock rate (`PLATFORM_CLOCK_HZ` in `viz/plot.py`, hardcoded from
  each `mrs-benchmark-<platform>/src/main.rs`'s clock config), so every page is in the same
  unit.
- Fixed 4x2 grid of task subplots per page, in a fixed order (`TASK_ORDER` in `viz/plot.py`,
  mirroring `inputs.json`'s case order). A task beyond one page's capacity spills onto a
  numbered continuation page (`Platform (page 2/2)`) instead of being dropped. A task present
  in the data but missing from `TASK_ORDER` is appended at the end with a stderr warning
  rather than silently disappearing.
- Every subplot on a platform's page reserves the same fixed set of library slots (that
  platform's libraries, in `LIBRARY_ORDER`), even for a task that doesn't implement one of
  them: the slot stays empty (bar omitted, tick label grayed/italic) so every subplot has
  identical width and the page reads as one consistent set.
- Each subplot: bar = median per (platform, task, library), overlaid with a jittered scatter of
  every raw per-iteration sample. Shows the real spread/density instead of a min/max whisker
  that a single freak sample (e.g. interrupt jitter) could stretch across the whole chart.
- Y-scale auto-flips to log when cross-library spread exceeds ~10x (only the transcendental
  tasks hit this); the y-axis label turns bold red on a log subplot so it can't be missed next
  to a linear neighbor.
- Fixed library -> color map; crazyflie-fw (the only C impl) gets a distinct hue plus a hatch and
  is always rightmost, so the C-vs-Rust comparison reads at a glance.
- Page header: platform name (bold), `n=` row count (italic subtitle). Per-subplot note when a
  library had errored inputs for that task.

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
- CI image carries the `uv` binary (multi-stage `COPY` in
  [Dockerfile.ci](../.gitlab/ci/Dockerfile.ci)); `UV_CACHE_DIR`/`UV_PYTHON_INSTALL_DIR` sit on
  `/persist` next to `CARGO_HOME` so wheels and the pinned CPython are fetched once.
- `cargo make report` runs the same command via `Makefile.toml`.

**Stable "latest `main`" link** (GitLab's latest-successful-artifact URL, no Pages needed):

https://git.tu-berlin.de/imrc/teaching/multi-robot-systems-project/2026/mrs-microbenchmarks/-/jobs/artifacts/main/raw/report.pdf?job=report-pdf

Relies on "Keep artifacts from most recent successful jobs" (confirmed on for this project) so
the link survives `expire_in`. TU login gating applies.

## Out of scope

- Cross-platform comparison on one page (times are already unified to `ns`; only the
  per-platform-page layout itself is out of scope).
- Trend-over-time charts.
- Interactive dashboard / Pages: revisit only if a static per-run PDF stops being enough.
- Paper-quality figures: a later local pass reusing `viz/plot.py`.
