# Benchmark Report CI: Design

Last updated: 2026-07-22

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
`expire_in: 1 year` ([collect.yml](../.gitlab/ci/collect.yml)). No new store.

Columns in [core `lib.rs`](../benchmarks/mrs-benchmark-core/src/lib.rs) (`CSV_HEADER`).
`duration` is total over the whole `repetitions` loop, not per-iteration.

### 2. `viz/plot.py`

Reads `results.csv`, writes the PDF, runs identically local and in CI. Plot design and deps
(matplotlib + pandas via uv) live in
[benchmark-viz-plotting-design.md](benchmark-viz-plotting-design.md).

### 3. CI changes

- New `report` stage, inserted between `collect` and `check` in `.gitlab-ci.yml`'s `stages:`
  list (and `include:`d from a new `.gitlab/ci/report.yml`, matching the one-file-per-stage
  split every other stage uses).
- **`report-pdf`** job:
  ```yaml
  report-pdf:
    stage: report
    needs:
      - job: collect-results
        artifacts: true
    allow_failure: true
    script:
      - uv run viz/plot.py results.csv report.pdf
    artifacts:
      name: "report-$CI_COMMIT_SHORT_SHA"
      paths:
        - report.pdf
      expire_in: 1 year
  ```
  `allow_failure: true` mirrors the HIL `run-*` jobs in [run.yml](../.gitlab/ci/run.yml): a
  plotting bug shouldn't block a merge. A separate job (not folded into `collect-results`) so a
  plotting failure can't fail `collect` and can be retried alone.
- **Custom CI image gains the `uv` binary** via a multi-stage `COPY` from the official image, in
  [Dockerfile.ci](../.gitlab/ci/Dockerfile.ci):
  ```dockerfile
  COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /usr/local/bin/
  ```
  Kaniko builds multi-stage Dockerfiles fine. Only the binary is baked, not the wheels — no
  checksum/install-script step needed.
- **uv state on `/persist`:** set `UV_CACHE_DIR=/persist/uv` and
  `UV_PYTHON_INSTALL_DIR=/persist/uv-python` next to the existing `CARGO_HOME` in
  `.gitlab-ci.yml`'s top-level `variables:`, so the matplotlib/pandas wheels and the pinned
  CPython uv bootstraps are fetched once, not every run. Same pattern as cargo, without
  cargo's rustup-baking special case (all uv state fits on `/persist`).
- **`Makefile.toml`** gets a `[tasks.report]` entry (`uv run viz/plot.py results.csv
  report.pdf`) alongside `bench-*`/`collect`, so `cargo make report` reaches it the same way as
  every other benchmark step.

### 4. Stable "latest `main`" link

GitLab's latest-successful-artifact URL bookmarks the newest `main` report, no Pages:

```
https://git.tu-berlin.de/imrc/teaching/multi-robot-systems-project/2026/mrs-microbenchmarks/-/jobs/artifacts/main/raw/report.pdf?job=report-pdf
```

- Always resolves to the latest successful `main` `report.pdf`. `raw/` previews inline;
  `download/` fetches; `browse?job=report-pdf` lists files.
- Relies on **"Keep artifacts from most recent successful jobs"** so the link survives
  `expire_in` — confirmed on for this project (2026-07-22).
- TU login gating applies, which is fine.

## Out of scope

- **Cross-platform comparison on one page:** needs cycle↔ns conversion (per chip clock). Later.
- **Trend-over-time charts:** deferred.
- **Interactive dashboard / Pages:** revisit only if a static per-run PDF stops being enough.
- **Paper-quality figures:** a later local pass reusing `viz/plot.py`.
