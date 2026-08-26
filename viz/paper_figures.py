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
from config import GROUP_GAP, LIBRARY_ORDER, LOG_COLOR, UNIT
from data import aggregate, choose_scale, load_accuracy_df, load_and_clean, load_library_versions, ordered
from pages_accuracy import MIN_HEADROOM_DECADES, ULP_LINTHRESH, ZERO_ULP_VISUAL_FLOOR

# One representative task per category (docs/task-categories.md): CrossProduct
# (arithmetic, free-ABI -- clean codegen comparison), MatMul3x3 (arithmetic,
# mat33 -- FFI-contaminated), Vec3Normalize (libm-bound), SinCos
# (transcendental, the libm-linking-bug task), MatMul9x9 (linalg via
# cmsis-dsp -- cleanest comparison in the suite).
CATEGORY_TASKS = ["CrossProduct", "MatMul3x3", "Vec3Normalize", "SinCos", "MatMul9x9"]

# SinCos's "crazyflie-fw" column calls sinf/cosf directly -- no crazyflie-firmware
# code executes (docs/crazyflie_math_integration.md, docs/task-categories.md's
# transcendental row). Relabeled here, in the paper's own curated figures only,
# so a bar doesn't imply a firmware algorithm is being measured; report.pdf's
# generic grid keeps "crazyflie-fw" since that's still the correct data key, and
# color/legend below stay tied to that real key too, only the tick text changes.
DISPLAY_LABEL_OVERRIDE = {("SinCos", "crazyflie-fw"): "newlib"}


def _display_labels(task, libs):
    return [DISPLAY_LABEL_OVERRIDE.get((task, lib), lib) for lib in libs]

# Cross-platform check: one clean-ABI task, one FFI-contaminated task.
CROSS_PLATFORM_TASKS = ["CrossProduct", "MatMul9x9"]
CROSS_PLATFORMS = ["stm32", "rp2350-arm", "esp32s3"]  # RP2040 excluded: no FPU, prose only

FLAGSHIP_PLATFORM = "stm32"
FLAGSHIP_PROFILE = "lto"


def _draw_time_panel(ax, subset, task, libs, task_samples, rng):
    """One subplot: bar+jitter for one task's per-library medians. Mirrors
    pages_time.py's plot_platform_page inner loop, standalone so callers can
    pick any small task/platform/profile combo instead of an exhaustive grid.
    Unlike the report grid, a library this task doesn't implement is simply
    left out rather than reserved as an empty greyed-out slot -- the paper's
    panels aren't part of one uniform multi-page set, so that consistency
    isn't worth the wasted width."""
    subset = subset.set_index("library")
    libs_present = [lib for lib in libs if lib in subset.index]
    medians = [subset.loc[lib, "median"] for lib in libs_present]
    scale = choose_scale(medians)

    x = list(range(len(libs_present)))
    task_n = int(subset["count"].max()) if libs_present else 0
    ax.set_title(f"{task} (n={task_n})", fontsize=9, pad=10)

    common.draw_bars(ax, x, medians, libs_present, width=0.6)
    for xi, lib in zip(x, libs_present):
        vals = task_samples.loc[task_samples["library"] == lib, "per_ns"]
        common.scatter_jitter(ax, xi, vals.tolist(), rng)

    if scale == "log":
        ax.set_yscale("log")
        ax.set_ylabel(f"{UNIT} (log)", fontsize=6.5, color=LOG_COLOR, fontweight="bold", labelpad=1)
    else:
        ax.set_ylabel(UNIT, fontsize=6.5, labelpad=1)
    ax.tick_params(axis="y", labelsize=6.5)
    ax.margins(y=0.9)  # bars stay well under half the panel height, not crowding it

    ax.set_xlim(-0.5, len(libs_present) - 0.5)
    ax.set_xticks(x)
    ax.set_xticklabels(_display_labels(task, libs_present), rotation=40, ha="right", fontsize=7)
    ax.tick_params(axis="x", pad=2)
    common.style_grid(ax)


def fig_category_grid(agg, df_ok, out_path):
    """Fig 1: execution time on the flagship platform/profile, one panel per
    task category."""
    rng = random.Random(0)
    tasks = [t for t in CATEGORY_TASKS if not agg[agg["task"] == t].empty]
    libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies)")

    fig, axes = plt.subplots(1, len(tasks), figsize=(7.16, 3.3))
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
    fig.suptitle(f"{FLAGSHIP_PLATFORM} [{FLAGSHIP_PROFILE}], one task per category", fontsize=9, y=1.02)
    fig.subplots_adjust(left=0.045, right=0.99, top=0.85, bottom=0.22, wspace=0.7)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def _draw_cross_platform_panel(ax, task, agg, df_ok, platforms, profile, libs, rng):
    """Groups are packed tight: a platform that skips a library (e.g. ESP32-S3
    skipping both C libraries) doesn't reserve that library's slot, so group
    width follows how many libraries actually ran there, not the full `libs`
    set -- this isn't part of the report's uniform multi-page grid, so there's
    nothing for a reserved-but-empty slot to stay consistent with."""
    task_rows = agg[agg["task"] == task]
    task_samples = df_ok[df_ok["task"] == task]

    tick_positions, tick_labels_text, group_bounds = [], [], []
    bar_x, bar_h, bar_libs = [], [], []
    cursor = 0.0

    for platform in platforms:
        plat_rows = task_rows[(task_rows["platform"] == platform) & (task_rows["profile"] == profile)].set_index("library")
        plat_samples = task_samples[(task_samples["platform"] == platform) & (task_samples["profile"] == profile)]
        libs_present = [lib for lib in libs if lib in plat_rows.index and pd.notna(plat_rows.loc[lib, "median"])]
        if not libs_present:
            continue

        if group_bounds:
            cursor += GROUP_GAP
        start = cursor
        for lib in libs_present:
            x = cursor
            vals = plat_samples.loc[plat_samples["library"] == lib, "per_ns"]
            common.scatter_jitter(ax, x, vals.tolist(), rng)
            bar_x.append(x)
            bar_h.append(plat_rows.loc[lib, "median"])
            bar_libs.append(lib)
            cursor += 1
        end = cursor - 1
        tick_positions.append((start + end) / 2)
        tick_labels_text.append(platform)
        group_bounds.append((start, end))

    common.shade_groups(ax, group_bounds)  # so each platform's group of bars reads at a glance

    scale = choose_scale(bar_h)
    common.draw_bars(ax, bar_x, bar_h, bar_libs, width=0.6)
    if scale == "log":
        ax.set_yscale("log")
        ax.set_ylabel(f"{UNIT} (log)", fontsize=7, color=LOG_COLOR, fontweight="bold", labelpad=1)
    else:
        ax.set_ylabel(UNIT, fontsize=7, labelpad=1)
    ax.tick_params(axis="y", labelsize=7)
    ax.margins(y=0.9)  # bars stay well under half the panel height, not crowding it

    task_n = int(task_rows["count"].max()) if not task_rows.empty else 0
    ax.set_title(f"{task} (n={task_n}) @ {profile}", fontsize=9, pad=8)
    ax.set_xlim(-0.5, (cursor - 1 if group_bounds else 0) + 0.5)
    ax.set_xticks(tick_positions)
    ax.set_xticklabels(tick_labels_text, fontsize=7, rotation=20, ha="right")
    ax.tick_params(axis="x", pad=2)
    common.style_grid(ax)
    return list(dict.fromkeys(bar_libs))  # unique, in libs order -- which libraries this panel actually drew


def fig_cross_platform(agg, df_ok, out_path):
    """Fig 2: does the flagship platform's per-task ranking hold on other
    platforms? One free-ABI task, one FFI-contaminated task, grouped by
    platform. RP2040 is left out here (no FPU -- covered in prose instead).
    Single-column width: as a full-textwidth figure* this read as
    disproportionately large next to the rest of the Results section. Side by
    side, not stacked -- tight per-platform grouping (no reserved slot for a
    library a platform skips) leaves enough width for both at this size."""
    rng = random.Random(1)
    libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies)")

    fig, axes = plt.subplots(1, len(CROSS_PLATFORM_TASKS), figsize=(3.4, 2.5))
    axes = axes.flatten()
    used_libs = []
    for ax, task in zip(axes, CROSS_PLATFORM_TASKS):
        for lib in _draw_cross_platform_panel(ax, task, agg, df_ok, CROSS_PLATFORMS, FLAGSHIP_PROFILE, libs, rng):
            if lib not in used_libs:
                used_libs.append(lib)

    # Legend only lists libraries these two tasks actually plot (4 of 6): the
    # full library set would pad the legend row wider than the panels above
    # it, forcing side whitespace once bbox_inches="tight" grows to fit it.
    common.add_legend(fig, used_libs, fontsize=6.5)
    # No fig.suptitle: at this width it only bought a tuning fight between its
    # y position and "top" for a near-zero gap; each panel's own title plus
    # the caption's "@ lto" already say what a suptitle would repeat.
    fig.subplots_adjust(left=0.16, right=0.98, top=0.90, bottom=0.24, wspace=0.6)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def _draw_accuracy_panel(ax, task, acc_df, samples_map, platform, profile, libs, rng):
    plat_acc = acc_df[(acc_df["platform"] == platform) & (acc_df["profile"] == profile)]
    task_acc = plat_acc[plat_acc["task"] == task]
    if task_acc.empty:
        ax.axis("off")
        return

    acc_indexed = task_acc.set_index("library")
    libs_present = [
        lib for lib in libs
        if lib in acc_indexed.index
        and pd.notna(acc_indexed.loc[lib, "mean_ulp"])
        and not math.isinf(acc_indexed.loc[lib, "mean_ulp"])
    ]
    if not libs_present:
        ax.axis("off")
        return

    x = list(range(len(libs_present)))
    ulps_present = [float(acc_indexed.loc[lib, "mean_ulp"]) for lib in libs_present]

    ax.set_title(f"{task} accuracy", fontsize=9, pad=10)
    ax.set_yscale("symlog", linthresh=ULP_LINTHRESH, linscale=1.0)

    bar_heights = [u if u > 0 else ZERO_ULP_VISUAL_FLOOR for u in ulps_present]
    max_y = max(max(ulps_present), ULP_LINTHRESH)
    top_y = 10.0 ** math.ceil(math.log10(max_y) + MIN_HEADROOM_DECADES)
    ax.set_ylim(0.0, top_y)

    common.draw_bars(ax, x, bar_heights, libs_present, bottom=0.0, width=0.6)
    for xi, lib in zip(x, libs_present):
        sample_list = samples_map.get((platform, profile, task, lib), [])
        vals = [u for u in sample_list if u is not None and not math.isnan(u) and not math.isinf(u)]
        if vals:
            common.scatter_jitter(ax, xi, vals, rng)

    ax.set_ylabel("Mean ULP (symlog)", fontsize=6.5, color=LOG_COLOR, fontweight="bold", labelpad=1)
    ax.tick_params(axis="y", labelsize=6.5)
    ax.set_xlim(-0.5, len(libs_present) - 0.5)
    ax.set_xticks(x)
    ax.set_xticklabels(_display_labels(task, libs_present), rotation=40, ha="right", fontsize=7)
    ax.tick_params(axis="x", pad=2)
    common.style_grid(ax)


def fig_accuracy(agg, acc_df, samples_map, df_ok, out_path):
    """Fig 3: ULP accuracy on the flagship platform/profile, same tasks as Fig 1
    -- the speed picture isn't free, this is what it costs."""
    rng = random.Random(2)
    tasks = [t for t in CATEGORY_TASKS if not agg[agg["task"] == t].empty]
    libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies)")

    fig, axes = plt.subplots(1, len(tasks), figsize=(7.16, 3.3))
    axes = axes.flatten()
    for ax, task in zip(axes, tasks):
        _draw_accuracy_panel(ax, task, acc_df, samples_map, FLAGSHIP_PLATFORM, FLAGSHIP_PROFILE, libs, rng)
    for ax in axes[len(tasks) :]:
        ax.axis("off")

    common.add_legend(fig, libs, fontsize=8)  # bare names: versions cited once in the caption, not per-figure
    fig.suptitle(f"{FLAGSHIP_PLATFORM} [{FLAGSHIP_PROFILE}] accuracy, same tasks as Fig. 1", fontsize=9, y=1.02)
    fig.subplots_adjust(left=0.045, right=0.99, top=0.85, bottom=0.22, wspace=0.7)
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
