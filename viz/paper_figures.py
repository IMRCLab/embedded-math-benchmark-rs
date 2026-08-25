"""Curated Results-section figures for the workshop paper: a small task/platform
subset instead of the full report grid. Reuses report.pdf's data loading and
chart primitives (data.py, common.py, config.py) -- plot.py/report.pdf itself
is untouched, this is an additional entry point, not a replacement.

Usage: uv run --project viz viz/paper_figures.py results.csv docs/draft/images/
"""

import math
import random
import sys

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
import pandas as pd

import common
from config import LIBRARY_ORDER, LOG_COLOR, UNIT
from data import aggregate, choose_scale, load_accuracy_df, load_and_clean, load_library_versions, ordered
from pages_accuracy import MIN_HEADROOM_DECADES, ULP_LINTHRESH, ZERO_ULP_VISUAL_FLOOR, _format_ulp

# One representative task per category (docs/task-categories.md): CrossProduct
# (arithmetic, free-ABI -- clean codegen comparison), MatMul3x3 (arithmetic,
# mat33 -- FFI-contaminated), Vec3Normalize (libm-bound), SinCos
# (transcendental, the libm-linking-bug task), MatMul9x9 (linalg via
# cmsis-dsp -- cleanest comparison in the suite).
CATEGORY_TASKS = ["CrossProduct", "MatMul3x3", "Vec3Normalize", "SinCos", "MatMul9x9"]

# Cross-platform check: one clean-ABI task, one FFI-contaminated task.
CROSS_PLATFORM_TASKS = ["CrossProduct", "MatMul9x9"]
CROSS_PLATFORMS = ["stm32", "rp2350-arm", "esp32s3"]  # RP2040 excluded: no FPU, prose only

FLAGSHIP_PLATFORM = "stm32"
FLAGSHIP_PROFILE = "lto"


def _draw_time_panel(ax, subset, task, libs, task_samples, rng):
    """One subplot: bar+jitter for one task's per-library medians. Mirrors
    pages_time.py's plot_platform_page inner loop, standalone so callers can
    pick any small task/platform/profile combo instead of an exhaustive grid."""
    subset = subset.set_index("library").reindex(libs)
    medians = subset["median"].tolist()
    present = [pd.notna(m) for m in medians]
    scale = choose_scale([m for m, p in zip(medians, present) if p])

    x = list(range(len(libs)))
    x_present = [xi for xi, p in zip(x, present) if p]
    medians_present = [m for m, p in zip(medians, present) if p]
    libs_present = [lib for lib, p in zip(libs, present) if p]

    task_n = int(subset["count"].max()) if present.count(True) else 0
    ax.set_title(f"{task} (n={task_n})", fontsize=9, pad=10)

    common.draw_bars(ax, x_present, medians_present, libs_present, width=0.6)
    for xi, lib in zip(x, libs):
        vals = task_samples.loc[task_samples["library"] == lib, "per_ns"]
        common.scatter_jitter(ax, xi, vals.tolist(), rng)

    if scale == "log":
        ax.set_yscale("log")
        ax.set_ylabel(f"{UNIT} (log)", fontsize=6.5, color=LOG_COLOR, fontweight="bold", labelpad=1)
    else:
        ax.set_ylabel(UNIT, fontsize=7, labelpad=1)
    ax.margins(y=0.15)
    for xi, median in zip(x_present, medians_present):
        common.label_bar_value(ax, xi, median, scale)

    ax.set_xlim(-0.5, len(libs) - 0.5)
    ax.set_xticks(x)
    tick_labels = ax.set_xticklabels(libs, rotation=30, ha="right", fontsize=7)
    common.style_absent_ticks(tick_labels, present)
    common.style_grid(ax)


def fig_category_grid(agg, df_ok, out_path):
    """Fig 1: execution time on the flagship platform/profile, one panel per
    task category."""
    rng = random.Random(0)
    tasks = [t for t in CATEGORY_TASKS if not agg[agg["task"] == t].empty]
    libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies)")

    fig, axes = plt.subplots(1, len(tasks), figsize=(7.16, 2.2))
    axes = axes.flatten()

    plat_rows = agg[(agg["platform"] == FLAGSHIP_PLATFORM) & (agg["profile"] == FLAGSHIP_PROFILE)]
    plat_samples = df_ok[(df_ok["platform"] == FLAGSHIP_PLATFORM) & (df_ok["profile"] == FLAGSHIP_PROFILE)]

    for ax, task in zip(axes, tasks):
        subset = plat_rows[plat_rows["task"] == task]
        task_samples = plat_samples[plat_samples["task"] == task]
        _draw_time_panel(ax, subset, task, libs, task_samples, rng)
    for ax in axes[len(tasks) :]:
        ax.axis("off")

    common.add_legend(fig, libs, fontsize=8)  # bare names: versions cited once in the caption, not per-figure
    fig.suptitle(f"{FLAGSHIP_PLATFORM} [{FLAGSHIP_PROFILE}], one task per category", fontsize=9, y=1.04)
    fig.subplots_adjust(left=0.045, right=0.99, top=0.8, bottom=0.4, wspace=0.8)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def _draw_cross_platform_panel(ax, task, agg, df_ok, platforms, profile, libs, rng):
    task_rows = agg[agg["task"] == task]
    task_samples = df_ok[df_ok["task"] == task]
    n_libs = len(libs)
    group_span = n_libs + 1.5

    tick_positions, tick_labels_text = [], []
    bar_x, bar_h, bar_libs = [], [], []

    for gi, platform in enumerate(platforms):
        base = gi * group_span
        tick_positions.append(base + (n_libs - 1) / 2)
        tick_labels_text.append(platform)

        plat_rows = (
            task_rows[(task_rows["platform"] == platform) & (task_rows["profile"] == profile)]
            .set_index("library")
            .reindex(libs)
        )
        plat_samples = task_samples[(task_samples["platform"] == platform) & (task_samples["profile"] == profile)]

        for li, lib in enumerate(libs):
            x = base + li
            vals = plat_samples.loc[plat_samples["library"] == lib, "per_ns"]
            common.scatter_jitter(ax, x, vals.tolist(), rng)
            median = plat_rows.loc[lib, "median"] if lib in plat_rows.index else float("nan")
            if pd.notna(median):
                bar_x.append(x)
                bar_h.append(median)
                bar_libs.append(lib)

    scale = choose_scale(bar_h)
    common.draw_bars(ax, bar_x, bar_h, bar_libs, width=0.6)
    if scale == "log":
        ax.set_yscale("log")
        ax.set_ylabel(f"{UNIT} (log)", fontsize=8, color=LOG_COLOR, fontweight="bold")
    else:
        ax.set_ylabel(UNIT, fontsize=8)
    ax.margins(y=0.15)

    task_n = int(task_rows["count"].max()) if not task_rows.empty else 0
    ax.set_title(f"{task} (n={task_n}) @ {profile}", fontsize=9, pad=10)
    n_groups = max(len(platforms), 1)
    ax.set_xlim(-0.5, (n_groups - 1) * group_span + (n_libs - 1) + 0.5)
    ax.set_xticks(tick_positions)
    ax.set_xticklabels(tick_labels_text, fontsize=8)
    common.style_grid(ax)


def fig_cross_platform(agg, df_ok, out_path):
    """Fig 2: does the flagship platform's per-task ranking hold on other
    platforms? One free-ABI task, one FFI-contaminated task, grouped by
    platform. RP2040 is left out here (no FPU -- covered in prose instead)."""
    rng = random.Random(1)
    libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies)")

    fig, axes = plt.subplots(1, len(CROSS_PLATFORM_TASKS), figsize=(7.16, 3.2))
    axes = axes.flatten()
    for ax, task in zip(axes, CROSS_PLATFORM_TASKS):
        _draw_cross_platform_panel(ax, task, agg, df_ok, CROSS_PLATFORMS, FLAGSHIP_PROFILE, libs, rng)

    common.add_legend(fig, libs, fontsize=8)  # bare names: versions cited once in the caption, not per-figure
    fig.suptitle(f"Cross-platform check @ [{FLAGSHIP_PROFILE}]", fontsize=9, y=1.02)
    fig.subplots_adjust(left=0.07, right=0.98, top=0.85, bottom=0.25, wspace=0.3)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def _draw_accuracy_panel(ax, task, acc_df, samples_map, platform, profile, libs, rng):
    plat_acc = acc_df[(acc_df["platform"] == platform) & (acc_df["profile"] == profile)]
    task_acc = plat_acc[plat_acc["task"] == task]
    if task_acc.empty:
        ax.axis("off")
        return

    acc_indexed = task_acc.set_index("library").reindex(libs)
    present = [
        lib in acc_indexed.index
        and pd.notna(acc_indexed.loc[lib, "mean_ulp"])
        and not math.isinf(acc_indexed.loc[lib, "mean_ulp"])
        for lib in libs
    ]
    x = list(range(len(libs)))
    x_present = [xi for xi, p in zip(x, present) if p]
    if not x_present:
        ax.axis("off")
        return

    ulps_present = [float(acc_indexed.loc[lib, "mean_ulp"]) for lib, p in zip(libs, present) if p]
    libs_present = [lib for lib, p in zip(libs, present) if p]

    ax.set_title(f"{task} accuracy", fontsize=9, pad=10)
    ax.set_yscale("symlog", linthresh=ULP_LINTHRESH, linscale=1.0)

    bar_heights = [u if u > 0 else ZERO_ULP_VISUAL_FLOOR for u in ulps_present]
    max_y = max(max(ulps_present), ULP_LINTHRESH)
    top_y = 10.0 ** math.ceil(math.log10(max_y) + MIN_HEADROOM_DECADES)
    ax.set_ylim(0.0, top_y)

    common.draw_bars(ax, x_present, bar_heights, libs_present, bottom=0.0, width=0.6)
    for xi, lib in zip(x_present, libs_present):
        sample_list = samples_map.get((platform, profile, task, lib), [])
        vals = [u for u in sample_list if u is not None and not math.isnan(u) and not math.isinf(u)]
        if vals:
            common.scatter_jitter(ax, xi, vals, rng)

    for xi, bh, u in zip(x_present, bar_heights, ulps_present):
        y = common.label_offset(bh, "symlog", ax, linthresh=ULP_LINTHRESH)
        common.draw_value_label(ax, xi, y, _format_ulp(u))

    ax.set_ylabel("Mean ULP (symlog)", fontsize=6, color=LOG_COLOR, fontweight="bold", labelpad=1)
    ax.set_xlim(-0.5, len(libs) - 0.5)
    ax.set_xticks(x)
    tick_labels = ax.set_xticklabels(libs, rotation=30, ha="right", fontsize=7)
    common.style_absent_ticks(tick_labels, present)
    common.style_grid(ax)


def fig_accuracy(agg, acc_df, samples_map, df_ok, out_path):
    """Fig 3: ULP accuracy on the flagship platform/profile, same tasks as Fig 1
    -- the speed picture isn't free, this is what it costs."""
    rng = random.Random(2)
    tasks = [t for t in CATEGORY_TASKS if not agg[agg["task"] == t].empty]
    libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies)")

    fig, axes = plt.subplots(1, len(tasks), figsize=(7.16, 2.2))
    axes = axes.flatten()
    for ax, task in zip(axes, tasks):
        _draw_accuracy_panel(ax, task, acc_df, samples_map, FLAGSHIP_PLATFORM, FLAGSHIP_PROFILE, libs, rng)
    for ax in axes[len(tasks) :]:
        ax.axis("off")

    common.add_legend(fig, libs, fontsize=8)  # bare names: versions cited once in the caption, not per-figure
    fig.suptitle(f"{FLAGSHIP_PLATFORM} [{FLAGSHIP_PROFILE}] accuracy, same tasks as Fig. 1", fontsize=9, y=1.04)
    fig.subplots_adjust(left=0.045, right=0.99, top=0.8, bottom=0.4, wspace=0.8)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def main(argv):
    csv_path, out_dir = argv[1], argv[2].rstrip("/")
    df_ok, _error_counts = load_and_clean(csv_path)
    agg = aggregate(df_ok)
    acc_df, samples_map = load_accuracy_df(csv_path)

    fig_category_grid(agg, df_ok, f"{out_dir}/fig-results-category.pdf")
    fig_cross_platform(agg, df_ok, f"{out_dir}/fig-results-crossplatform.pdf")
    if acc_df is not None:
        fig_accuracy(agg, acc_df, samples_map, df_ok, f"{out_dir}/fig-results-accuracy.pdf")
    else:
        print("paper_figures.py: no accuracy data found, skipping fig-results-accuracy.pdf", file=sys.stderr)

    # Figure legends show bare library names (long "lib @version" strings ate
    # too much width on a paper-sized figure) -- print the versions once here
    # so they're easy to copy into the paper's caption/prose instead.
    versions = load_library_versions(csv_path)
    print(f"paper_figures.py: library versions for the caption: {versions}", file=sys.stderr)
    print(f"paper_figures.py: wrote figures to {out_dir}/", file=sys.stderr)


if __name__ == "__main__":
    main(sys.argv)
