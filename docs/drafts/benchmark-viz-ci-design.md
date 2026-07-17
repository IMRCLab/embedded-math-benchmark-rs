# Benchmark Report CI: Design

Last updated: 2026-07-16

## Problem

Compare **platform × library × task × input** on the latest run, mostly during development.
2-3 person team, milestone in ~6 weeks.

Out for now: history/trend views, interactive filtering, cross-run comparison in one view
(GitLab keeps per-pipeline artifacts already).

## Approach

CI emits a **PDF report as a pipeline artifact**. No dashboard, hosting, or service.

```
run-host / run-mcu ─▶ collect-results ─▶ report-pdf
                      (results.csv)        (viz/plot.py → report.pdf)
```

A plain script that runs the same locally and in CI, so it seeds the later paper-figure pass
instead of being throwaway. Plot design: [benchmark-viz-plotting-design.md](benchmark-viz-plotting-design.md).

## Components

### 1. `results.csv` (already durable)

`collect-results` merges `BENCH ` rows into `results.csv` and uploads it with
`expire_in: 1 year` ([collect.yml](../../.gitlab/ci/collect.yml)). No new store.

Columns in [core `lib.rs`](../../benchmarks/mrs-benchmark-core/src/lib.rs) (`CSV_HEADER`).
`duration` is total over the whole `repetitions` loop, not per-iteration.

### 2. `viz/plot.py`

Reads `results.csv`, writes the PDF, runs identically local and in CI. Plot design and deps
(matplotlib + pandas via uv) live in
[benchmark-viz-plotting-design.md](benchmark-viz-plotting-design.md).

### 3. CI changes

- New `report` stage after `collect`.
- **`report-pdf`** job: `needs: collect-results`, runs `uv run viz/plot.py results.csv
  report.pdf`, uploads `report.pdf`. A separate job so a plotting failure can't fail `collect`
  and can be retried alone.
- Custom CI image gains the `uv` binary (baked alongside the C/Rust deps in
  [image.yml](../../.gitlab/ci/image.yml)). Only the binary is baked, not the wheels.
- **uv state on `/persist`:** set `UV_CACHE_DIR=/persist/uv` and
  `UV_PYTHON_INSTALL_DIR=/persist/uv-python` next to the existing `CARGO_HOME`, so the
  matplotlib/pandas wheels and the pinned CPython uv bootstraps are fetched once, not every
  run. Same pattern as cargo, without cargo's rustup-baking special case (all uv state fits on
  `/persist`).

### 4. Stable "latest `main`" link

GitLab's latest-successful-artifact URL bookmarks the newest `main` report, no Pages:

```
https://git.tu-berlin.de/imrc/teaching/multi-robot-systems-project/2026/mrs-microbenchmarks/-/jobs/artifacts/main/raw/report.pdf?job=report-pdf
```

- Always resolves to the latest successful `main` `report.pdf`. `raw/` previews inline;
  `download/` fetches; `browse?job=report-pdf` lists files.
- Enable **"Keep artifacts from most recent successful jobs"** (on by default) so it survives
  `expire_in`.
- TU login gating applies, which is fine.

## Out of scope

- **Cross-platform comparison on one page:** needs cycle↔ns conversion (per chip clock). Later.
- **Trend-over-time charts:** deferred.
- **Interactive dashboard / Pages:** revisit only if a static per-run PDF stops being enough.
- **Paper-quality figures:** a later local pass reusing `viz/plot.py`.
