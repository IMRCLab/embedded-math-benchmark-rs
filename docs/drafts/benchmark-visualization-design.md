# Benchmark Result Visualization: Design

Last updated: 2026-07-07

## Problem

Goal: a dashboard to compare across **platform × library × task × input**, mostly on the
latest run. Picking a past run by branch/commit is nice to have. History-over-time trends
are **not** a priority now.

Constraints:

- Host on GitLab Pages (`git.tu-berlin.de`) if possible. No VPS, no persistent service to
  babysit.
- Audience is the 2-3 person team. TU login gating is fine; timing data isn't sensitive.
- Paper-quality figures are out of scope here (done later via a local Python pass over the
  same input).

## Approach

Two pipelines sharing one data branch:

```
CI (per commit, all branches)          Pages job (all branches)
─────────────────────────────          ────────────────────────
build → run → collect (csv)            checkout ci/bench-data
  → emit run.json                        → copy runs/ + index.json → public/data/
  → push to ci/bench-data                → copy dashboard → public/
     (+ update index.json)               → deploy Pages
```

- **`ci/bench-data`**: orphan branch. One small JSON per run created, append-only. Durable source of truth; survives the dashboard being deleted.
- **Pages site**: a disposable view, rebuildable from `ci/bench-data` at any time.

Static was chosen over MLflow because the dashboard is just a chart spec served as files.
Nothing to keep running. One-time cost to write the spec beats the recurring cost of owning a service.

## Components

### 1. `ci/bench-data` orphan branch

- Created once manually(`git checkout --orphan ci/bench-data`)
- Holds only:
  - `runs/<ts>-<branch>-<shortsha>.json`, one per run (schema below). The name sorts
    roughly by time and is human-scannable.
  - `index.json`, a metadata-only manifest of all runs.
- No `.gitlab-ci.yml`, so pushing to it triggers no pipeline.

### 2. Per-run JSON

`runs/1751900000-main-abcdef1.json`:

```json
{
  "commit_sha": "abcdef123456...",
  "commit_short_sha": "abcdef1",
  "commit_ts": 1751900000,
  "branch": "main",
  "pipeline_id": "123456",
  "rows": [
    {
      "platform": "host",
      "library": "glam",
      "task": "MatMul3x3",
      "input_index": 0,
      "repetitions": 1000,
      "duration": 104523,
      "unit": "ns"
    }
  ]
}
```

- `rows` is the parsed `results.csv`, minus the `result` column (correctness data, not
  needed for charts).
- `commit_ts` (`git show -s --format=%ct`) is recorded now so a later history view could sort by git time. Unused today, cheap to keep.

### 3. `index.json`

```json
{
  "runs": [
    {
      "file": "1751900000-main-abcdef1.json",
      "short_sha": "abcdef1",
      "branch": "main",
      "ts": 1751900000
    }
  ]
}
```

Metadata only, no rows. It exists because static Pages has no directory listing, so the
browser needs a list of run files. The dashboard reads it to fill the run picker, then
fetches only the selected run.

### 4. CI changes

**`mrs-benchmark-collect`** ([crate](../../../benchmarks/mrs-benchmark-collect)) gets a JSON output mode reusing its existing `BENCH ` parser, so no second parser and the pipeline stays Rust-only.

**Push job** (after `collect-results`, all branches):

1. Fetch/checkout `ci/bench-data`.
2. Write the new `runs/*.json`, append its entry to `index.json`.
3. Optional Commit and push via a **fetch → rebase → retry loop** so close-together pipelines don't race and drop a run.
4. Pushes with `GITLAB_ACCESS_TOKEN` (already available as a masked CI variable,
   `write_repository` scope).

**Pages job** (all branches, `needs:` the push job so this run's file exists first):

1. Checkout `ci/bench-data`.
2. Copy `runs/` and `index.json` into `public/data/`.
3. Copy the dashboard into `public/`.
4. Deploy.

The site depends only on `ci/bench-data`, so any branch's pages job rebuilds the same thing.
Running on every branch means you see feature-branch results right away, at the cost of extra
runner time.

### 5. Dashboard

One static page, no build step, charts via
[Observable Plot](https://observablehq.com/plot/).

Obersavle Plot vs Vega-Lite: both facet well, but the page is an imperative loop (pick run → fetch
JSON → filter by checkboxes → re-render), which fits Plot's "call `Plot.plot()` again with
new data" model. Vega-Lite's self-contained reactive spec fights external HTML controls.
Chart.js/uPlot were out (no built-in faceting). Plot's charts are JS not a portable spec,
but paper figures are redone in Python anyway.

Vendor Plot + d3 under `public/vendor/` instead of a CDN script: no external host at view
time (works offline, fine with Pages Access Control), at the cost of a manual version bump.

Layout:

- **Run picker** (top): dropdown from `index.json`, defaults to latest `main` run.
- **Filters**: plain checkboxes for platform/library/task.
- **Chart**: faceted bars, one facet per task, colored by library, **faceted by platform**. Host is `ns`, MCUs are `cycles`, so platforms must never share an axis. Faceting enforces that.

## Edge cases

- **Concurrent pushes**: handled by the fetch → rebase → retry loop.
- **No `BENCH` rows**: `collect` already exits non-zero; the push job then writes no run file.
- **Repo growth**: hundreds of small JSON files is trivial for git.

## Out of scope

- **Trend-over-time charts**: deferred. `commit_ts` is stored so this needs no data migration later.
- **MLflow**: revisit only if fixed charts stop being enough.
- **Python paper figures**: optional, loads the same JSON locally when needed.
