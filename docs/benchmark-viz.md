# Benchmark report

`viz/plot.py` renders `results.csv` into a multipage `report.pdf`, comparing platform x library x
task x input on the latest run. CI publishes it as a pipeline artifact.

Stable "latest `main`" link:
https://git.tu-berlin.de/imrc/teaching/multi-robot-systems-project/2026/mrs-microbenchmarks/-/jobs/artifacts/main/raw/report.pdf?job=report-pdf

```bash
uv run --project viz viz/plot.py results.csv report.pdf   # same command locally and in CI
cargo make report                                          # wrapper for the above
glab job artifact $(git branch --show-current) report-pdf  # fetch CI's copy
```

## Report structure

Page 1 is a table of contents. Section 1 is one page-set per platform x compile profile, section 2
transposes that to one subplot per task with the x-axis grouped by platform, section 3 is accuracy
per platform x profile, and section 4 is a Pareto speed/accuracy table placed after each
platform's own accuracy pages. `viz/toc.py` inserts the TOC and a nested bookmark tree afterwards
with `pypdf`, as a pure post-process over page numbers `plot.py` records.

Firmware cycle counts convert to ns via `PLATFORM_CLOCK_HZ` in `viz/config.py`, which is why a new
platform has to be registered there. Bars are the median per (platform, task, library) with the
raw samples jittered over them. The y-axis auto-flips to log when cross-library spread exceeds
~10x, flagged with a bold red axis label so a log panel is not mistaken for a linear neighbour.
Legend entries append each library's version from `library_versions.json` when present (e.g.
`glam 0.30.10`, `crazyflie-fw @54f31e2`), falling back to the bare name.

Accuracy pages use a symlog axis: linear below 1 ULP so a bit-exact result gets a real position,
log above it. Mark, scale and colour details are in `viz/pages_time.py` and
`viz/pages_accuracy.py`.

`viz/` is a uv project: `config.py` (constants), `data.py` (load/transform), `common.py` (shared
chart chrome), `pages_time.py`/`pages_accuracy.py` (page builders), `toc.py`, `plot.py` (CLI).
Pages render in a `ProcessPoolExecutor`, which cuts a ~110-page report from ~35s to ~9s on 12
cores. There is no automated test: a plotting script's correctness is visual, so verify by running
against a real `results.csv` and reading the PDF.

## Paper figures

`viz/paper_figures.py` is a second entry point for the workshop paper, not part of the CI report.
It imports `data.py`/`common.py`/`config.py` but draws its own small grids sized for an IEEE
two-column figure.

```bash
uv run --project viz viz/paper_figures.py results.csv docs/draft/images/
```

It writes `fig-results-{category,profile,crossplatform}.pdf` and prints the accuracy table's LaTeX
rows plus resolved library versions to stderr. Three conventions differ from the report:

- **One profile per panel, not one per figure.** `CATEGORY_TASKS` maps task to profile, so each
  ABI class is drawn where its C-vs-Rust number is valid
  ([task-categories.md](task-categories.md#which-profile-makes-a-c-vs-rust-number-valid)). Each
  panel prints its own `@ profile`.
- **Cross-platform panels are normalized to each group's fastest library.** Absolute cycles differ
  ~40x between the M0+ and the M4F, and a shared log axis flattens the within-platform ordering
  those panels exist to show.
- **Accuracy is a table, not a figure.** Mean ULP spans 0.07 to 1.2e5. `ACCURACY_EXCLUDE` drops
  `crazyflie-fw`'s composite row, which is divergence rather than error
  ([accuracy_evaluation.md](accuracy_evaluation.md#the-composite-reference-is-not-neutral)).

## CI

Job `report-pdf` in [.gitlab/ci/report.yml](../.gitlab/ci/report.yml) runs in its own `report`
stage between `collect` and `check`, with `allow_failure: true` so a plotting bug cannot block a
merge and can be retried alone. The CI image carries the `uv` binary, with `UV_CACHE_DIR` and
`UV_PYTHON_INSTALL_DIR` on `/persist` next to `CARGO_HOME`.
