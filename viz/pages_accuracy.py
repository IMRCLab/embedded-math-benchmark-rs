"""Accuracy (ULP) bar-chart pages (Section 3) and the per-platform speed/accuracy
summary table (Section 4)."""

import math
import random

import matplotlib.pyplot as plt
import pandas as pd

import common
from config import GRID_SHAPE, LOG_COLOR
from data import compute_pareto_frontier

# log10 gap reserved above the tallest bar for its value label. Comfortably more
# than common.LOG_LABEL_MULTIPLIER's own offset needs (log10(1.18) ~= 0.07
# decades) -- the old headroom formula (`max_y * 1.8`, rounded up to the next
# power of ten) could land as little as ~0.27 decades above a bar that fell just
# under a decade boundary, which is why labels used to get clipped by the axis's
# hard top edge or crowd the subplot title above it.
MIN_HEADROOM_DECADES = 0.5

# Below this many ULP, the y-axis is linear instead of log (symlog): keeps 0 ULP
# (bit-exact) a real, distinct position instead of the old "+1" bar-height hack
# that made every bar's drawn height disagree with its printed label.
ULP_LINTHRESH = 1.0

# Minimum drawn bar height for an exact (0 ULP) result, in linthresh units --
# a literal 0-height bar is visually indistinguishable from "library not run"
# (which also draws no bar). Purely cosmetic: the printed label still reads the
# true value ("0 (exact)"), only the rectangle gets a floor.
ZERO_ULP_VISUAL_FLOOR = 0.03 * ULP_LINTHRESH

# Verdict text colors for the Pareto summary table -- deliberately not reused
# from LIBRARY_COLORS so a colored library name in these two columns can't be
# misread as "this is that library's usual chart color".
PARETO_COLOR = "#1e7a34"
DOMINATED_COLOR = "#8a8a8a"


def _format_ulp(u):
    """Compact ULP value text -- no ' ULP' suffix, the axis label already says
    'Mean ULP + 1 (log)', and repeating it on every bar was most of why labels
    collided with their neighbors."""
    if u == 0.0:
        return "0 (exact)"
    if u < 10.0:
        return f"{u:.1f}"
    if u < 1000.0:
        return f"{u:.0f}"
    return f"{u:.1e}"


def plot_accuracy_platform_page(acc_df, samples_map, platform, profile, libs, tasks, page, n_pages, generated_at, versions=None):
    """Plots per-platform accuracy bar charts (Mean ULP) for each task, matching time bar chart styling."""
    n_rows, n_cols = GRID_SHAPE
    fig, axes = plt.subplots(n_rows, n_cols, figsize=(16, 8))
    axes = axes.flatten()
    # Applied before the subplot loop (not after, as the time-chart pages do) because
    # the symlog value-label placement below reads each axes' pixel geometry via
    # ax.transData -- computing that against the default pre-adjustment layout gave
    # a gap sized for a different subplot size than what actually renders.
    fig.subplots_adjust(left=0.055, right=0.985, top=0.89, bottom=0.13, hspace=0.7, wspace=0.35)

    plat_acc = (
        acc_df[(acc_df["platform"] == platform) & (acc_df["profile"] == profile)]
        if acc_df is not None
        else None
    )

    has_any_plot = False
    x = list(range(len(libs)))
    rng = random.Random(0)

    for ax, task in zip(axes, tasks):
        if plat_acc is None:
            ax.axis("off")
            continue

        task_acc = plat_acc[plat_acc["task"] == task]
        if task_acc.empty:
            ax.axis("off")
            continue

        acc_indexed = task_acc.set_index("library").reindex(libs)

        present = [
            lib in acc_indexed.index
            and pd.notna(acc_indexed.loc[lib, "mean_ulp"])
            and not math.isinf(acc_indexed.loc[lib, "mean_ulp"])
            for lib in libs
        ]

        x_present = [xi for xi, p in zip(x, present) if p]
        if not x_present:
            ax.axis("off")
            continue

        has_any_plot = True
        ulps_present = [float(acc_indexed.loc[lib, "mean_ulp"]) for lib, p in zip(libs, present) if p]
        libs_present = [lib for lib, p in zip(libs, present) if p]

        ax.set_title(f"{task} Accuracy", fontsize=9, pad=14)
        ax.set_yscale("symlog", linthresh=ULP_LINTHRESH, linscale=1.0)

        # Bars plot the real ULP value directly -- symlog's own linear region near
        # zero (up to ULP_LINTHRESH) gives 0 ULP a real, defined position without
        # the old "+1 to every bar" offset that made bar height disagree with the
        # printed label. A true-zero bar still gets a small visual floor (see
        # ZERO_ULP_VISUAL_FLOOR) so it stays visible instead of a zero-height
        # rectangle indistinguishable from "library not run".
        bar_heights = [u if u > 0 else ZERO_ULP_VISUAL_FLOOR for u in ulps_present]
        max_y = max(max(ulps_present), ULP_LINTHRESH)
        exp_headroom = math.ceil(math.log10(max_y) + MIN_HEADROOM_DECADES)
        top_y = 10.0 ** exp_headroom
        ax.set_ylim(0.0, top_y)

        common.draw_bars(ax, x_present, bar_heights, libs_present, bottom=0.0, width=0.6)

        # Jittered raw ULP samples over each bar: matches execution time bar chart styling
        for xi, lib in zip(x_present, libs_present):
            sample_list = samples_map.get((platform, profile, task, lib), [])
            if sample_list:
                vals = [u for u in sample_list if u is not None and not math.isnan(u) and not math.isinf(u)]
                if vals:
                    common.scatter_jitter(ax, xi, vals, rng)

        for xi, bh, u in zip(x_present, bar_heights, ulps_present):
            y = common.label_offset(bh, "symlog", ax, linthresh=ULP_LINTHRESH)
            common.draw_value_label(ax, xi, y, _format_ulp(u))

        ax.set_ylabel("Mean ULP (symlog)", fontsize=8, color=LOG_COLOR, fontweight="bold")

        ax.set_xlim(-0.5, len(libs) - 0.5)
        ax.set_xticks(x)
        tick_labels = ax.set_xticklabels(libs, rotation=30, ha="right", fontsize=8)
        common.style_absent_ticks(tick_labels, present)
        common.style_grid(ax)

    if not has_any_plot:
        plt.close(fig)
        return None

    for ax in axes[len(tasks):]:
        ax.axis("off")

    title = f"{platform} [{profile}] Accuracy (Mean ULP Error)"
    title = title if n_pages == 1 else f"{title} (page {page}/{n_pages})"
    subtitle = (
        "ULP (Unit in the Last Place): Float32 LSB bit distance. "
        "0 ULP = Bit-exact match, 1-2 ULP = IEEE float noise"
    )
    common.page_header(fig, title, subtitle, generated_at)
    common.add_legend(fig, libs, versions)
    return fig


def plot_pareto_summary_table_page(agg, acc_df, platform, profile, tasks, generated_at):
    """Renders a clean Pareto Classification Summary Table for a platform at the end of the report."""
    plat_agg = agg[(agg["platform"] == platform) & (agg["profile"] == profile)]
    plat_acc = (
        acc_df[(acc_df["platform"] == platform) & (acc_df["profile"] == profile)]
        if acc_df is not None
        else None
    )
    if plat_agg.empty or plat_acc is None:
        return None

    table_data = []
    for task in tasks:
        sub_agg = plat_agg[plat_agg["task"] == task]
        sub_acc = plat_acc[plat_acc["task"] == task]
        merged = pd.merge(sub_agg, sub_acc, on=["platform", "profile", "library", "task"], how="inner")
        if merged.empty:
            continue

        points = [(row["library"], row["median"], row["mean_ulp"]) for _, row in merged.iterrows()]
        if not points:
            continue

        fastest = min(points, key=lambda p: p[1])
        valid_acc = [p for p in points if not pd.isna(p[2]) and not math.isinf(p[2])]

        if valid_acc:
            min_ulp_val = min(p[2] for p in valid_acc)
            most_acc_libs = [p for p in valid_acc if abs(p[2] - min_ulp_val) < 0.1]
            names = "/".join(sorted(p[0] for p in most_acc_libs))
            most_acc_str = (
                f"{names} ({min_ulp_val:.1f} ULP)" if min_ulp_val < 1000.0
                else f"{names} ({min_ulp_val:.1e} ULP)"
            )
        else:
            most_acc_str = "N/A"

        pareto_pts = compute_pareto_frontier(points)
        pareto_libs = {p[0] for p in pareto_pts}
        all_libs = {p[0] for p in points}
        dominated_libs = sorted(all_libs - pareto_libs)

        fastest_str = f"{fastest[0]} ({fastest[1]:.1f} ns)"
        pareto_str = ", ".join(sorted(pareto_libs))
        dominated_str = ", ".join(dominated_libs) if dominated_libs else "None"

        table_data.append([task, fastest_str, most_acc_str, pareto_str, dominated_str])

    if not table_data:
        return None

    fig, ax = plt.subplots(figsize=(16, 8))
    ax.axis("off")
    # Reserve the top of the figure for the title/subtitle text below, and
    # confine the table to an explicit bbox (axes fraction) rather than letting
    # it size itself from row count + a fixed .scale() multiplier -- that grew
    # taller than the axes and overlapped the subtitle once enough tasks (and
    # so table rows) accumulated.
    fig.subplots_adjust(top=0.83, bottom=0.04, left=0.03, right=0.97)

    headers = ["Task", "Fastest Library", "Most Accurate Library", "Pareto Optimal Set", "Dominated Libraries"]
    table = ax.table(cellText=table_data, colLabels=headers, cellLoc="center", loc="center", bbox=[0, 0, 1, 1])
    table.auto_set_font_size(False)
    table.set_fontsize(9)

    for i in range(len(headers)):
        table[(0, i)].set_facecolor("#2a78d6")
        table[(0, i)].set_text_props(color="white", fontweight="bold")

    # Color the verdict columns so "worth using" vs "skip it" reads at a glance
    # instead of requiring the column headers to be re-read for every row.
    pareto_col, dominated_col = headers.index("Pareto Optimal Set"), headers.index("Dominated Libraries")
    for row_idx, row in enumerate(table_data, start=1):
        table[(row_idx, pareto_col)].get_text().set_color(PARETO_COLOR)
        table[(row_idx, pareto_col)].get_text().set_fontweight("bold")
        if row[dominated_col] != "None":
            table[(row_idx, dominated_col)].get_text().set_color(DOMINATED_COLOR)

    fig.suptitle(f"{platform} [{profile}] - Speed & Accuracy Trade-off Summary", fontsize=16, fontweight="bold", y=0.96)
    fig.text(
        0.5, 0.90,
        "ULP (Unit in the Last Place): Measures float32 LSB bit distance (~1.19e-7 step at magnitude 1.0).\n"
        "0 ULP = Bit-exact float32 match | 1-2 ULP = Float noise | >10 ULP = Drift / Approximation.\n"
        "Pareto optimal = no other library is both faster and more accurate on this task. "
        "Dominated = at least one library beats it on both.",
        ha="center", va="top", fontsize=9, style="italic", color="#444444",
    )
    fig.text(0.99, 0.99, f"generated {generated_at}", ha="right", va="top", fontsize=8, color="#555555")
    return fig
