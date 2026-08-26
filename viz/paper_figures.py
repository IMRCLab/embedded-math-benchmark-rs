"""Curated Results-section figures for the workshop paper: a small task/platform
subset instead of the full report grid. Reuses report.pdf's data loading and
chart primitives (data.py, common.py, config.py). plot.py/report.pdf itself is
untouched, this is an additional entry point, not a replacement.

Usage: uv run --project viz viz/paper_figures.py results.csv docs/draft/images/
"""

import math
import random
import sys

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
import pandas as pd
from matplotlib.ticker import NullLocator

import common
from config import GROUP_GAP, LIBRARY_ORDER, LOG_COLOR, PROFILE_ORDER, UNIT
from data import aggregate, choose_scale, load_accuracy_df, load_and_clean, load_library_versions, ordered

# One task per ABI class, each read at the profile where its C-vs-Rust number is
# valid (docs/task-categories.md, "Which profile makes a C-vs-Rust number
# valid"). Mixing profiles within one figure is deliberate: at any single
# profile some of these rows measure struct marshalling or a missing LTO pass
# rather than generated math.
CATEGORY_TASKS = {
    "CrossProduct": "lto",   # free ABI: args fit in registers, nothing to marshal
    "MatMul3x3": "xlto",     # mat33 by-value: only cross-language LTO removes the AAPCS copy
    "MatMul9x9": "xlto",     # pointer ABI, but C only gets inlined at xlto
    "SinCos": "lto",         # transcendental provider, insensitive to profile
}

# SinCos's and Sqrt's "crazyflie-fw" column calls sinf/cosf/sqrtf directly, so no
# crazyflie-firmware code executes (docs/crazyflie_math_integration.md,
# docs/task-categories.md's transcendental row). Relabeled here, in the paper's own
# curated figures only, so a bar doesn't imply a firmware algorithm is being
# measured; report.pdf's generic grid keeps "crazyflie-fw" since that's still the
# correct data key, and color/legend below stay tied to that real key too, only the
# tick text changes.
DISPLAY_LABEL_OVERRIDE = {
    ("SinCos", "crazyflie-fw"): "newlib",
    ("Sqrt", "crazyflie-fw"): "newlib",
}

# Cross-platform check: does the flagship's ordering survive a different core?
# CrossProduct carries the parity claim, Sqrt the transcendental-provider claim.
# Both at lto, the one profile every platform builds (xlto is ARM hard-float only).
CROSS_PLATFORM_TASKS = ["CrossProduct", "Sqrt"]
CROSS_PLATFORMS = ["stm32", "rp2350-arm", "rp2040", "esp32s3"]

# Profile sensitivity: C / fastest-Rust for one task per ABI class, swept over
# every profile. The only figure where a task appears at more than one profile,
# and the one place the verdict flips.
RATIO_TASKS = ["CrossProduct", "MatMul3x3", "MatMul9x9", "DotProduct64D"]
C_LIBRARIES = ["crazyflie-fw", "cmsis-dsp"]
RATIO_COLORS = ["#2a78d6", "#eb6834", "#1baf7a", "#8e44ad"]

# Accuracy is reported as a LaTeX table, not a figure: mean ULP spans 0.07 to
# 1.2e5 here, which reads as numbers but not as bars.
ACCURACY_TASKS = ["Sqrt", "SinCos", "QuatSlerp", "LeeController"]
# crazyflie-fw's composite ULP is divergence from a different algorithm, not error
# (docs/accuracy_evaluation.md, "The composite reference is not neutral").
ACCURACY_EXCLUDE = {("LeeController", "crazyflie-fw")}

FLAGSHIP_PLATFORM = "stm32"
FLAGSHIP_PROFILE = "lto"


def _legend_below(fig, libs, fontsize, gap=0.02):
    """common.add_legend anchors at the figure's lower edge, which is inside the
    bottom margin the rotated tick labels hang into. Anchoring the legend's top
    below that edge keeps the two apart regardless of how long a label is."""
    fig.legend(
        handles=common.legend_handles(libs), loc="upper center",
        bbox_to_anchor=(0.5, -gap), ncol=len(libs), frameon=False,
        fontsize=fontsize, handlelength=1.2, handleheight=0.9, columnspacing=1.0,
    )


def _display_labels(task, libs):
    return [DISPLAY_LABEL_OVERRIDE.get((task, lib), lib) for lib in libs]


def _draw_time_panel(ax, subset, task, profile, libs, task_samples, rng):
    """One subplot: bar+jitter for one task's per-library medians. Mirrors
    pages_time.py's plot_platform_page inner loop, standalone so callers can
    pick any small task/platform/profile combo instead of an exhaustive grid.
    Unlike the report grid, a library this task doesn't implement is simply
    left out rather than reserved as an empty greyed-out slot: the paper's
    panels aren't part of one uniform multi-page set, so that consistency
    isn't worth the wasted width."""
    subset = subset.set_index("library")
    libs_present = [lib for lib in libs if lib in subset.index]
    medians = [subset.loc[lib, "median"] for lib in libs_present]
    scale = choose_scale(medians)

    x = list(range(len(libs_present)))
    task_n = int(subset["count"].max()) if libs_present else 0
    ax.set_title(f"{task} (n={task_n})", fontsize=8.5, pad=11)
    # Per-panel profile: the figure mixes them on purpose, so it has to read off the panel.
    ax.text(0.5, 1.02, f"@ {profile}", transform=ax.transAxes, ha="center", va="bottom",
            fontsize=7, color="#444444", fontstyle="italic")

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
    # margins are fractional of the axis range, and on a log axis that range is
    # in decades, so the report's 0.9 buries paper-sized bars under headroom.
    ax.margins(y=0.25 if scale == "log" else 0.9)

    ax.set_xlim(-0.5, len(libs_present) - 0.5)
    ax.set_xticks(x)
    ax.set_xticklabels(_display_labels(task, libs_present), rotation=45, ha="right", fontsize=6.2)
    ax.tick_params(axis="x", pad=2)
    common.style_grid(ax)


def fig_category_grid(agg, df_ok, out_path):
    """Fig 1: execution time on the flagship platform, one panel per ABI class,
    each at the profile where that class's comparison is valid."""
    rng = random.Random(0)
    tasks = [t for t in CATEGORY_TASKS if not agg[agg["task"] == t].empty]
    libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies)")

    fig, axes = plt.subplots(1, len(tasks), figsize=(7.16, 1.7))
    axes = axes.flatten()

    for ax, task in zip(axes, tasks):
        profile = CATEGORY_TASKS[task]
        sel = (agg["platform"] == FLAGSHIP_PLATFORM) & (agg["profile"] == profile) & (agg["task"] == task)
        sel_s = (df_ok["platform"] == FLAGSHIP_PLATFORM) & (df_ok["profile"] == profile) & (df_ok["task"] == task)
        _draw_time_panel(ax, agg[sel], task, profile, libs, df_ok[sel_s], rng)
    for ax in axes[len(tasks):]:
        ax.axis("off")

    _legend_below(fig, libs, fontsize=8)  # bare names: versions cited once in the caption, not per-figure
    fig.suptitle(f"{FLAGSHIP_PLATFORM}, one task per ABI class", fontsize=9, y=1.05)
    fig.subplots_adjust(left=0.045, right=0.99, top=0.80, bottom=0.28, wspace=0.75)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def _c_over_rust(agg, task, platform, profile):
    """C median / fastest-Rust median, or None if either side is missing. Fastest
    Rust rather than a named crate: which crate wins varies by task, and pinning
    one would make the ratio a statement about that crate instead of about C."""
    rows = agg[(agg["task"] == task) & (agg["platform"] == platform) & (agg["profile"] == profile)]
    if rows.empty:
        return None
    c_rows = rows[rows["library"].isin(C_LIBRARIES)]
    rust_rows = rows[~rows["library"].isin(C_LIBRARIES)]
    if c_rows.empty or rust_rows.empty:
        return None
    return float(c_rows["median"].min()) / float(rust_rows["median"].min())


def fig_profile_ratio(agg, out_path):
    """Fig 2: C / fastest-Rust across all four build profiles. Above 1.0 Rust is
    ahead, below it C is; the point is that a single task crosses the line."""
    profiles = [p for p in PROFILE_ORDER if p in set(agg["profile"])]
    fig, ax = plt.subplots(figsize=(3.4, 1.95))

    n = len(RATIO_TASKS)
    width = 0.8 / n
    for i, task in enumerate(RATIO_TASKS):
        xs, ys = [], []
        for j, profile in enumerate(profiles):
            ratio = _c_over_rust(agg, task, FLAGSHIP_PLATFORM, profile)
            if ratio is None:
                continue
            xs.append(j - 0.4 + width * (i + 0.5))
            ys.append(ratio)
        bars = ax.bar(xs, ys, width=width * 0.9, color=RATIO_COLORS[i % len(RATIO_COLORS)],
                      label=task, zorder=2)
        for bar, y in zip(bars, ys):
            ax.text(bar.get_x() + bar.get_width() / 2, y * 1.03, f"{y:.2f}", rotation=90,
                    ha="center", va="bottom", fontsize=5.2, color="#222222", zorder=5)

    ax.axhline(1.0, color="#111111", linewidth=0.9, linestyle="--", zorder=3)
    ax.text(-0.46, 1.04, "parity", fontsize=6, color="#111111", ha="left", va="bottom")
    ax.set_yscale("log")
    ax.set_ylim(0.5, 5.2)
    ax.set_yticks([0.6, 1.0, 2.0, 3.0])
    ax.set_yticklabels(["0.6", "1.0", "2.0", "3.0"], fontsize=6.5)
    ax.yaxis.set_minor_locator(NullLocator())  # a sub-decade log axis otherwise sprouts a "4x10^0" label
    ax.set_ylabel("C / fastest Rust (log)", fontsize=6.5, color=LOG_COLOR, fontweight="bold", labelpad=1)
    ax.set_xticks(range(len(profiles)))
    ax.set_xticklabels(profiles, fontsize=7.5)
    ax.set_xlim(-0.5, len(profiles) - 0.5)
    ax.tick_params(axis="x", pad=2)
    common.shade_groups(ax, [(j, j) for j in range(len(profiles))])
    common.style_grid(ax)
    ax.legend(fontsize=6, ncol=4, frameon=False, loc="upper center",
              bbox_to_anchor=(0.5, -0.13), columnspacing=0.8, handlelength=1.0)
    fig.subplots_adjust(left=0.15, right=0.99, top=0.96, bottom=0.28)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def _draw_cross_platform_panel(ax, task, agg, task_medians, platforms, profile, libs):
    """Each platform group is normalized to its own fastest library. Absolute
    cycles differ by ~40x between the M0+ and the M4F, and on a shared log axis
    that gap flattens every within-platform ordering into a flat row of equal
    bars, which is the one thing this panel exists to show.

    Groups are packed tight: a platform that skips a library (e.g. ESP32-S3
    skipping both C libraries) doesn't reserve that library's slot, so group
    width follows how many libraries actually ran there, not the full `libs`
    set. This isn't part of the report's uniform multi-page grid, so there's
    nothing for a reserved-but-empty slot to stay consistent with."""
    tick_positions, tick_labels_text, group_bounds = [], [], []
    bar_x, bar_h, bar_libs = [], [], []
    cursor = 0.0

    for platform in platforms:
        plat_rows = task_medians[(task_medians["platform"] == platform) & (task_medians["profile"] == profile)].set_index("library")
        libs_present = [lib for lib in libs if lib in plat_rows.index and pd.notna(plat_rows.loc[lib, "median"])]
        if not libs_present:
            continue
        fastest = min(float(plat_rows.loc[lib, "median"]) for lib in libs_present)

        if group_bounds:
            cursor += GROUP_GAP
        start = cursor
        for lib in libs_present:
            bar_x.append(cursor)
            bar_h.append(float(plat_rows.loc[lib, "median"]) / fastest)
            bar_libs.append(lib)
            cursor += 1
        end = cursor - 1
        tick_positions.append((start + end) / 2)
        tick_labels_text.append(platform)
        group_bounds.append((start, end))

    common.shade_groups(ax, group_bounds)  # so each platform's group of bars reads at a glance
    common.draw_bars(ax, bar_x, bar_h, bar_libs, width=0.7)
    ax.axhline(1.0, color="#111111", linewidth=0.7, linestyle="--", zorder=3)

    ax.set_ylabel("x fastest on platform", fontsize=6.5, labelpad=1)
    ax.set_ylim(0, max(bar_h) * 1.18 if bar_h else 1)
    ax.tick_params(axis="y", labelsize=6.5)

    task_n = int(task_medians["count"].max()) if not task_medians.empty else 0
    ax.set_title(f"{task} (n={task_n}) @ {profile}", fontsize=8, pad=6)
    ax.set_xlim(-0.5, (cursor - 1 if group_bounds else 0) + 0.5)
    ax.set_xticks(tick_positions)
    ax.set_xticklabels(tick_labels_text, rotation=20, ha="right", fontsize=6.5)
    ax.tick_params(axis="x", pad=2)
    common.style_grid(ax)
    return list(dict.fromkeys(bar_libs))  # unique, in libs order: which libraries this panel actually drew


def fig_cross_platform(agg, df_ok, out_path):
    """Fig 3: does the flagship platform's per-task ranking hold on other cores?
    One free-ABI task, one transcendental task, grouped by platform, each group
    normalized to its own fastest library. Single-column width: as a
    full-textwidth figure* this read as disproportionately large next to the
    rest of the Results section."""
    libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies)")

    fig, axes = plt.subplots(1, len(CROSS_PLATFORM_TASKS), figsize=(3.4, 1.95))
    axes = axes.flatten()
    used_libs = []
    for ax, task in zip(axes, CROSS_PLATFORM_TASKS):
        for lib in _draw_cross_platform_panel(ax, task, agg, agg[agg["task"] == task], CROSS_PLATFORMS, FLAGSHIP_PROFILE, libs):
            if lib not in used_libs:
                used_libs.append(lib)

    # Legend only lists libraries these two tasks actually plot: the full
    # library set would pad the legend row wider than the panels above it,
    # forcing side whitespace once bbox_inches="tight" grows to fit it.
    _legend_below(fig, used_libs, fontsize=6.5, gap=0.0)
    fig.subplots_adjust(left=0.16, right=0.98, top=0.90, bottom=0.24, wspace=0.6)
    fig.savefig(out_path, bbox_inches="tight")
    plt.close(fig)


def _fmt_ulp(value):
    if value is None or (isinstance(value, float) and (math.isnan(value) or math.isinf(value))):
        return "n/a"
    if value < 10:
        return f"{value:.2f}"
    return f"{value:,.0f}".replace(",", "{,}")


def accuracy_table(acc_df, libs):
    """LaTeX body rows for the paper's mean-ULP table. Emitted rather than
    plotted: the values span six decades, which reads as numbers, not bars."""
    plat = acc_df[(acc_df["platform"] == FLAGSHIP_PLATFORM) & (acc_df["profile"] == FLAGSHIP_PROFILE)]
    lines = []
    for task in ACCURACY_TASKS:
        rows = plat[plat["task"] == task].set_index("library")
        cells = []
        for lib in libs:
            if lib not in rows.index or (task, lib) in ACCURACY_EXCLUDE:
                cells.append("n/a")
            else:
                cells.append(_fmt_ulp(float(rows.loc[lib, "mean_ulp"])))
        lines.append(f"\\texttt{{{task}}} & " + " & ".join(cells) + r" \\")
    return lines


def main(argv):
    csv_path, out_dir = argv[1], argv[2].rstrip("/")
    df_ok, _error_counts = load_and_clean(csv_path)
    agg = aggregate(df_ok)
    acc_df, _samples_map = load_accuracy_df(csv_path)

    fig_category_grid(agg, df_ok, f"{out_dir}/fig-results-category.pdf")
    fig_profile_ratio(agg, f"{out_dir}/fig-results-profile.pdf")
    fig_cross_platform(agg, df_ok, f"{out_dir}/fig-results-crossplatform.pdf")

    if acc_df is not None:
        libs = ordered(acc_df["library"].unique(), LIBRARY_ORDER, "library(ies)")
        print("paper_figures.py: accuracy table body (mean ULP, "
              f"{FLAGSHIP_PLATFORM} @ {FLAGSHIP_PROFILE}), columns: {libs}", file=sys.stderr)
        for line in accuracy_table(acc_df, libs):
            print("  " + line, file=sys.stderr)
    else:
        print("paper_figures.py: no accuracy data found, skipping the ULP table", file=sys.stderr)

    # Figure legends show bare library names (long "lib @version" strings ate
    # too much width on a paper-sized figure), so print the versions once here
    # for the paper's caption/prose instead.
    versions = load_library_versions(csv_path)
    print(f"paper_figures.py: library versions for the caption: {versions}", file=sys.stderr)
    print(f"paper_figures.py: wrote figures to {out_dir}/", file=sys.stderr)


if __name__ == "__main__":
    main(sys.argv)
