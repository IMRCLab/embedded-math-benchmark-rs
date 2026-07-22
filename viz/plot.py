# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "matplotlib",
#     "pandas",
# ]
# ///
"""Render results.csv into a per-platform multipage PDF report.

Usage: uv run viz/plot.py results.csv report.pdf
"""

import random
import sys

import matplotlib.pyplot as plt
import pandas as pd
from matplotlib.backends.backend_pdf import PdfPages
from matplotlib.patches import Patch

# Fixed order + validated colorblind-safe palette (dataviz skill, categorical
# slots 1-5). crazyflie-fw is the only C impl: distinct hue, always rightmost,
# plus a hatch so it reads as "different" even in grayscale/print.
LIBRARY_ORDER = ["glam", "libm", "micromath", "nalgebra", "crazyflie-fw"]
LIBRARY_COLORS = {
    "glam": "#2a78d6",
    "libm": "#eb6834",
    "micromath": "#1baf7a",
    "nalgebra": "#eda100",
    "crazyflie-fw": "#e87ba4",
}
CF_HATCH = "///"
LOG_THRESHOLD = 10.0
GRID_SHAPE = (2, 4)


def load_and_clean(csv_path):
    """Split ok/error rows, add per-iteration duration to the ok rows.

    Returns (df_ok, error_counts) where error_counts maps
    (platform, task, library) -> number of ERROR rows.
    """
    df = pd.read_csv(csv_path)
    is_error = df["result"].astype(str).str.startswith("ERROR")

    df_ok = df[~is_error].copy()
    df_ok["per"] = df_ok["duration"] / df_ok["repetitions"]

    error_counts = (
        df[is_error].groupby(["platform", "task", "library"]).size().to_dict()
    )
    return df_ok, error_counts


def aggregate(df_ok):
    """Per (platform, task, library): median/count of `per`."""
    return (
        df_ok.groupby(["platform", "task", "library"])["per"]
        .agg(median="median", count="count")
        .reset_index()
    )


def choose_scale(medians):
    """Log if cross-library spread of medians exceeds LOG_THRESHOLD, else linear."""
    if len(medians) < 2:
        return "linear"
    lo, hi = min(medians), max(medians)
    if lo <= 0:
        return "linear"
    return "log" if (hi / lo) > LOG_THRESHOLD else "linear"


def _task_note(task_errors):
    """One short line naming libraries that had errored inputs for this task."""
    if not task_errors:
        return ""
    parts = [f"{lib}: {n} err" for lib, n in sorted(task_errors.items())]
    return "\n" + ", ".join(parts)


def plot_platform_page(agg, df_ok, error_counts, platform, unit, tasks):
    n_rows, n_cols = GRID_SHAPE
    fig, axes = plt.subplots(n_rows, n_cols, figsize=(16, 8))
    axes = axes.flatten()

    platform_rows = agg[agg["platform"] == platform]
    total_n = int(platform_rows["count"].sum())
    platform_samples = df_ok[df_ok["platform"] == platform]
    rng = random.Random(0)

    for ax, task in zip(axes, tasks):
        subset = platform_rows[platform_rows["task"] == task]
        if subset.empty:
            ax.axis("off")
            continue

        subset = subset.set_index("library").reindex(
            [lib for lib in LIBRARY_ORDER if lib in subset["library"].values]
        )
        libs = list(subset.index)
        medians = subset["median"].tolist()
        scale = choose_scale(medians)

        task_errors = {
            lib: n
            for (p, t, lib), n in error_counts.items()
            if p == platform and t == task
        }
        ax.set_title(f"{task}{' (log)' if scale == 'log' else ''}{_task_note(task_errors)}", fontsize=9)

        task_samples = platform_samples[platform_samples["task"] == task]
        colors = [LIBRARY_COLORS[lib] for lib in libs]
        x = list(range(len(libs)))

        bars = ax.bar(x, medians, color=colors, width=0.6, zorder=2)
        for bar, lib in zip(bars, libs):
            if lib == "crazyflie-fw":
                bar.set_hatch(CF_HATCH)

        # Jittered raw samples over each bar: shows the real spread/density
        # instead of a single min/max whisker that one freak sample can stretch.
        for xi, lib in zip(x, libs):
            vals = task_samples.loc[task_samples["library"] == lib, "per"]
            xs = [xi + rng.uniform(-0.18, 0.18) for _ in range(len(vals))]
            ax.scatter(xs, vals, s=6, color="#0b0b0b", alpha=0.35, zorder=3, linewidths=0)

        if scale == "log":
            ax.set_yscale("log")

        ax.set_xticks(list(x))
        ax.set_xticklabels(libs, rotation=30, ha="right", fontsize=8)
        ax.set_ylabel(unit, fontsize=8)
        ax.grid(axis="y", color="#d0d0d0", linewidth=0.6, zorder=0)
        ax.set_axisbelow(True)

    for ax in axes[len(tasks):]:
        ax.axis("off")

    fig.suptitle(platform, fontsize=18, fontweight="bold", y=0.99)
    fig.text(
        0.5, 0.95,
        f"n={total_n} rows",
        ha="center", fontsize=9, fontstyle="italic", color="#444444",
    )

    handles = [
        Patch(
            facecolor=LIBRARY_COLORS[lib],
            hatch=CF_HATCH if lib == "crazyflie-fw" else None,
            label=lib,
        )
        for lib in LIBRARY_ORDER
    ]
    fig.legend(handles=handles, loc="lower center", ncol=len(LIBRARY_ORDER), frameon=False)
    fig.tight_layout(rect=(0, 0.05, 1, 0.92))
    return fig


def main(argv):
    csv_path, out_pdf = argv[1], argv[2]
    df_ok, error_counts = load_and_clean(csv_path)
    agg = aggregate(df_ok)
    units = df_ok.groupby("platform")["unit"].first()
    tasks = sorted(df_ok["task"].unique())

    with PdfPages(out_pdf) as pdf:
        for platform in sorted(agg["platform"].unique()):
            fig = plot_platform_page(agg, df_ok, error_counts, platform, units[platform], tasks)
            pdf.savefig(fig)
            plt.close(fig)


if __name__ == "__main__":
    main(sys.argv)
