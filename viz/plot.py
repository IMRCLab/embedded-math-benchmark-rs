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

import math
import os
import random
import sys
from datetime import datetime

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
LIBRARY_MARKERS = {
    "glam": "o",
    "libm": "s",
    "micromath": "^",
    "nalgebra": "D",
    "crazyflie-fw": "v",
}
CF_HATCH = "///"
FALLBACK_COLOR = "#999999"

# Display order mirrors benchmarks/inputs.json case order. Kept by hand (like
# LIBRARY_ORDER above) rather than read from inputs.json: a results.csv may be
# reviewed without the Rust tree next to it, and collect's merged CSV rows are
# already alphabetically sorted so there's no order left to recover from the
# data itself. A task present in the data but missing here is appended
# (alphabetically) rather than silently dropped.
TASK_ORDER = [
    "MatMul3x3",
    "RotateVector",
    "MatInverse3x3",
    "Atan2",
    "SinCos",
    "Sqrt",
    "QuatMul",
    "UnitQuatMul",
    "QuatSlerp",
    "LeeController",
]

# host first, then MCUs in CLAUDE.md's crate map order.
PLATFORM_ORDER = ["host", "stm32", "rp2040", "rp2350-arm", "esp32s3"]

LOG_THRESHOLD = 10.0
LOG_COLOR = "#c62828"
GRID_SHAPE = (2, 4)  # rows, cols per page; extra tasks spill onto page 2, 3, ...
TASK_GRID_SHAPE = (2, 2)  # by-task pages: fewer, wider subplots for the platform x library groups
UNIT = "ns"

# Cycle counters read raw core cycles; converting to time needs each
# platform's clock rate (see the `main.rs` of the corresponding
# mrs-benchmark-<platform> crate for where each is configured). host already
# reports `ns` directly via std::time::Instant and needs no conversion.
PLATFORM_CLOCK_HZ = {
    "stm32": 168e6,
    "rp2040": 125e6,
    "rp2350-arm": 150e6,
    "esp32s3": 240e6,
}


def _ordered(present, canonical_order, what):
    """canonical_order entries present in the data, in order, then any
    unlisted ones appended (sorted), with a warning so additions to
    inputs.json don't silently fall out of the plot."""
    present = set(present)
    known = [x for x in canonical_order if x in present]
    unknown = sorted(present - set(canonical_order))
    if unknown:
        print(f"plot.py: {what} not listed, appending at the end: {unknown}", file=sys.stderr)
    return known + unknown


def to_nanoseconds(df_ok):
    """Convert the `per` column (already per-iteration) to nanoseconds.

    host reports `ns` natively. Firmware platforms report raw core cycles;
    dividing by that platform's clock rate makes every platform's page
    comparable in the same real-time unit instead of raw cycle counts.
    """
    is_cycles = df_ok["unit"] == "cycles"
    missing = sorted(set(df_ok.loc[is_cycles, "platform"]) - set(PLATFORM_CLOCK_HZ))
    if missing:
        sys.exit(
            f"plot.py: no clock rate configured for platform(s) {missing} in "
            "PLATFORM_CLOCK_HZ -- add one before results can be converted to time"
        )
    hz = df_ok["platform"].map(PLATFORM_CLOCK_HZ)
    return df_ok["per"].where(~is_cycles, df_ok["per"] / hz * 1e9)


def load_and_clean(csv_path):
    """Split ok/error rows, add per-iteration duration (ns) to the ok rows.

    Returns (df_ok, error_counts) where error_counts maps
    (platform, task, library) -> number of ERROR rows.
    """
    df = pd.read_csv(csv_path)
    is_error = df["result"].astype(str).str.startswith("ERROR")

    df_ok = df[~is_error].copy()
    df_ok["per"] = df_ok["duration"] / df_ok["repetitions"]
    df_ok["per_ns"] = to_nanoseconds(df_ok)

    error_counts = (
        df[is_error].groupby(["platform", "task", "library"]).size().to_dict()
    )
    return df_ok, error_counts


def aggregate(df_ok):
    """Per (platform, task, library): median/count of `per_ns`."""
    return (
        df_ok.groupby(["platform", "task", "library"])["per_ns"]
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
    return ", ".join(parts)


def _paginate(items, page_size):
    return [items[i : i + page_size] for i in range(0, len(items), page_size)]


def _scatter_jitter(ax, x_center, values, rng):
    xs = [x_center + rng.uniform(-0.18, 0.18) for _ in range(len(values))]
    ax.scatter(xs, values, s=6, color="#0b0b0b", alpha=0.35, zorder=3, linewidths=0)


def _format_compact(value):
    return f"{value / 1000:.2f}k" if value >= 1000 else f"{value:.0f}"


def _label_bar_value(ax, x_center, bar_top, scale, rotation=0):
    # Anchored to bar_top (not jitter) so it reads as this bar's label.
    if scale == "log":
        y = bar_top * 1.18
    else:
        lo, hi = ax.get_ylim()
        y = bar_top + 0.045 * (hi - lo)
    ax.text(
        x_center, y, _format_compact(bar_top), rotation=rotation,
        ha="center", va="bottom", fontsize=6.5, color="#222222", zorder=5,
        bbox=dict(boxstyle="round,pad=0.05", facecolor="white", edgecolor="none", alpha=0.45),
    )


def plot_platform_page(
    agg, df_ok, error_counts, platform, libs, tasks, page, n_pages, generated_at
):
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

        # Reindex to the platform's full library set (not just this task's),
        # so every subplot on the page has identical width/spacing. Libraries
        # this task doesn't implement show up as a reserved, empty slot.
        subset = subset.set_index("library").reindex(libs)
        medians = subset["median"].tolist()
        present = [pd.notna(m) for m in medians]
        scale = choose_scale([m for m, p in zip(medians, present) if p])

        task_errors = {
            lib: n
            for (p, t, lib), n in error_counts.items()
            if p == platform and t == task
        }
        # pad=14 leaves room for the error note below the title; max (not
        # equality) since an errored-out library has a lower count.
        task_n = int(subset["count"].max())
        ax.set_title(f"{task} (n={task_n})", fontsize=9, pad=14)
        note = _task_note(task_errors)
        if note:
            ax.text(
                0.5, 1.0, note,
                transform=ax.transAxes, ha="center", va="bottom",
                fontsize=6, color="#999999",
            )

        task_samples = platform_samples[platform_samples["task"] == task]
        x = list(range(len(libs)))
        colors = [LIBRARY_COLORS.get(lib, FALLBACK_COLOR) for lib in libs]

        x_present = [xi for xi, p in zip(x, present) if p]
        medians_present = [m for m, p in zip(medians, present) if p]
        colors_present = [c for c, p in zip(colors, present) if p]
        libs_present = [lib for lib, p in zip(libs, present) if p]

        bars = ax.bar(x_present, medians_present, color=colors_present, width=0.6, zorder=2)
        for bar, lib in zip(bars, libs_present):
            if lib == "crazyflie-fw":
                bar.set_hatch(CF_HATCH)

        # Jittered raw samples over each bar: shows the real spread/density
        # instead of a single min/max whisker that one freak sample can stretch.
        for xi, lib in zip(x, libs):
            vals = task_samples.loc[task_samples["library"] == lib, "per_ns"]
            _scatter_jitter(ax, xi, vals.tolist(), rng)

        if scale == "log":
            ax.set_yscale("log")
            ax.set_ylabel(f"{UNIT} (log)", fontsize=8, color=LOG_COLOR, fontweight="bold")
        else:
            ax.set_ylabel(UNIT, fontsize=8)
        ax.margins(y=0.15)  # headroom for the value label above the tallest bar

        for xi, median, lib in zip(x_present, medians_present, libs_present):
            _label_bar_value(ax, xi, median, scale)

        # Fixed x-extent regardless of how many bars actually drew, so every
        # task subplot on the page reads as part of the same set.
        ax.set_xlim(-0.5, len(libs) - 0.5)
        ax.set_xticks(x)
        tick_labels = ax.set_xticklabels(libs, rotation=30, ha="right", fontsize=8)
        for label, p in zip(tick_labels, present):
            if not p:
                label.set_color("#aaaaaa")
                label.set_style("italic")

        ax.grid(axis="y", color="#d0d0d0", linewidth=0.6, zorder=0)
        ax.set_axisbelow(True)

    for ax in axes[len(tasks):]:
        ax.axis("off")

    title = platform if n_pages == 1 else f"{platform} (page {page}/{n_pages})"
    fig.suptitle(title, fontsize=18, fontweight="bold", y=0.99)
    fig.text(
        0.5, 0.94,
        f"n={total_n} rows",
        ha="center", fontsize=9, fontstyle="italic", color="#444444",
    )
    fig.text(
        0.99, 0.99,
        f"generated {generated_at}",
        ha="right", va="top", fontsize=9, color="#555555",
    )

    handles = [
        Patch(
            facecolor=LIBRARY_COLORS.get(lib, FALLBACK_COLOR),
            hatch=CF_HATCH if lib == "crazyflie-fw" else None,
            label=lib,
        )
        for lib in libs
    ]
    fig.legend(handles=handles, loc="lower center", ncol=len(libs), frameon=False)
    # Fixed margins, not tight_layout: tight_layout sizes cells from the tight
    # bbox of all axes including ones turned off, so a sparse continuation
    # page got extra padding and its one subplot visibly inflated.
    fig.subplots_adjust(left=0.055, right=0.985, top=0.86, bottom=0.13, hspace=0.7, wspace=0.35)
    return fig


# Empty space (in slot-widths) between platform groups within a task subplot.
GROUP_GAP = 2.5


def plot_task_page(agg, df_ok, error_counts, tasks, platforms, libs, page, n_pages, generated_at):
    """plot_platform_page transposed: one subplot per task, x-axis grouped by
    platform, bars within each group by library. Every group reserves the
    full `libs` set so all groups on the page are the same width."""
    n_rows, n_cols = TASK_GRID_SHAPE
    fig, axes = plt.subplots(n_rows, n_cols, figsize=(16, 8))
    axes = axes.flatten()

    total_n = int(agg[agg["task"].isin(tasks)]["count"].sum())
    rng = random.Random(0)
    n_libs = len(libs)
    group_span = n_libs + GROUP_GAP

    for ax, task in zip(axes, tasks):
        task_rows = agg[agg["task"] == task]
        if task_rows.empty:
            ax.axis("off")
            continue
        task_platforms = [p for p in platforms if p in set(task_rows["platform"])]
        task_samples = df_ok[df_ok["task"] == task]

        task_n = int(task_rows["count"].max())
        ax.set_title(f"{task} (n={task_n})", fontsize=9, pad=14)

        # A per-(platform, library) breakdown (like plot_platform_page's note)
        # would run off the subplot at this bar count; total is enough here,
        # the per-platform pages already carry the itemized breakdown.
        task_error_total = sum(n for (p, t, lib), n in error_counts.items() if t == task)
        if task_error_total:
            ax.text(
                0.5, 1.0, f"{task_error_total} errored input(s) across platforms",
                transform=ax.transAxes, ha="center", va="bottom",
                fontsize=6, color="#999999",
            )

        tick_positions = []
        tick_labels_text = []
        bars_info = []  # (x, median, lib)

        for gi, platform in enumerate(task_platforms):
            base = gi * group_span
            tick_positions.append(base + (n_libs - 1) / 2)
            tick_labels_text.append(platform)

            plat_rows = task_rows[task_rows["platform"] == platform].set_index("library").reindex(libs)
            plat_samples = task_samples[task_samples["platform"] == platform]

            for li, lib in enumerate(libs):
                x = base + li
                vals = plat_samples.loc[plat_samples["library"] == lib, "per_ns"]
                _scatter_jitter(ax, x, vals.tolist(), rng)
                median = plat_rows.loc[lib, "median"]
                if pd.notna(median):
                    bars_info.append((x, median, lib))

        scale = choose_scale([m for _, m, _ in bars_info])
        bar_x = [x for x, _, _ in bars_info]
        bar_h = [m for _, m, _ in bars_info]
        bar_c = [LIBRARY_COLORS.get(lib, FALLBACK_COLOR) for _, _, lib in bars_info]
        bars = ax.bar(bar_x, bar_h, color=bar_c, width=0.6, zorder=2)
        for bar, (_, _, lib) in zip(bars, bars_info):
            if lib == "crazyflie-fw":
                bar.set_hatch(CF_HATCH)

        if scale == "log":
            ax.set_yscale("log")
            ax.set_ylabel(f"{UNIT} (log)", fontsize=8, color=LOG_COLOR, fontweight="bold")
        else:
            ax.set_ylabel(UNIT, fontsize=8)
        ax.margins(y=0.1)

        n_groups = max(len(task_platforms), 1)
        ax.set_xlim(-0.5, (n_groups - 1) * group_span + (n_libs - 1) + 0.5)
        ax.set_xticks(tick_positions)
        ax.set_xticklabels(tick_labels_text, rotation=30, ha="right", fontsize=8)

        ax.grid(axis="y", color="#d0d0d0", linewidth=0.6, zorder=0)
        ax.set_axisbelow(True)

    for ax in axes[len(tasks):]:
        ax.axis("off")

    title = "All platforms, grouped by task"
    title = title if n_pages == 1 else f"{title} (page {page}/{n_pages})"
    fig.suptitle(title, fontsize=18, fontweight="bold", y=0.99)
    fig.text(
        0.5, 0.94,
        f"n={total_n} rows",
        ha="center", fontsize=9, fontstyle="italic", color="#444444",
    )
    fig.text(
        0.99, 0.99,
        f"generated {generated_at}",
        ha="right", va="top", fontsize=9, color="#555555",
    )

    handles = [
        Patch(
            facecolor=LIBRARY_COLORS.get(lib, FALLBACK_COLOR),
            hatch=CF_HATCH if lib == "crazyflie-fw" else None,
            label=lib,
        )
        for lib in libs
    ]
    fig.legend(handles=handles, loc="lower center", ncol=len(libs), frameon=False)
    fig.subplots_adjust(left=0.055, right=0.985, top=0.86, bottom=0.13, hspace=0.7, wspace=0.1)
    return fig


def main(argv):
    csv_path, out_pdf = argv[1], argv[2]
    df_ok, error_counts = load_and_clean(csv_path)
    agg = aggregate(df_ok)
    tasks = _ordered(df_ok["task"].unique(), TASK_ORDER, "task(s)")
    page_size = GRID_SHAPE[0] * GRID_SHAPE[1]
    task_pages = _paginate(tasks, page_size)
    by_task_pages = _paginate(tasks, TASK_GRID_SHAPE[0] * TASK_GRID_SHAPE[1])
    generated_at = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    all_platforms = _ordered(agg["platform"].unique(), PLATFORM_ORDER, "platform(s)")
    all_libs = _ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies) globally")

    with PdfPages(out_pdf) as pdf:
        for platform in all_platforms:
            platform_libs = _ordered(
                df_ok.loc[df_ok["platform"] == platform, "library"].unique(),
                LIBRARY_ORDER,
                f"library(ies) on {platform}",
            )
            for page_idx, task_page in enumerate(task_pages, start=1):
                page_rows = agg[
                    (agg["platform"] == platform) & (agg["task"].isin(task_page))
                ]
                if page_rows.empty:
                    continue
                fig = plot_platform_page(
                    agg, df_ok, error_counts, platform, platform_libs, task_page,
                    page_idx, len(task_pages), generated_at,
                )
                pdf.savefig(fig)
                plt.close(fig)

        # Section 2: Execution Time (by task)
        for page_idx, task_page in enumerate(by_task_pages, start=1):
            page_rows = agg[agg["task"].isin(task_page)]
            if page_rows.empty:
                continue
            fig = plot_task_page(
                agg, df_ok, error_counts, task_page, all_platforms, all_libs,
                page_idx, len(by_task_pages), generated_at,
            )
            pdf.savefig(fig)
            plt.close(fig)

        # Section 3: Accuracy Bar Charts (per platform)
        acc_df = load_accuracy_df(csv_path)
        if acc_df is not None:
            for platform in all_platforms:
                platform_libs = _ordered(
                    df_ok.loc[df_ok["platform"] == platform, "library"].unique(),
                    LIBRARY_ORDER,
                    f"library(ies) on {platform}",
                )
                for page_idx, task_page in enumerate(task_pages, start=1):
                    fig = plot_accuracy_platform_page(
                        acc_df, platform, platform_libs, task_page,
                        page_idx, len(task_pages), generated_at,
                    )
                    if fig is not None:
                        pdf.savefig(fig)
                        plt.close(fig)

            # Section 4: Accuracy & Speed Comparison Table (at the very end of report.pdf)
            for platform in all_platforms:
                summary_fig = plot_pareto_summary_table_page(agg, acc_df, platform, tasks, generated_at)
                if summary_fig is not None:
                    pdf.savefig(summary_fig)
                    plt.close(summary_fig)


def load_accuracy_df(csv_path):
    """Loads accuracy_results.csv if present in the workspace root."""
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    acc_csv = os.path.join(root_dir, "accuracy_results.csv")
    if os.path.exists(acc_csv):
        try:
            return pd.read_csv(acc_csv)
        except Exception as e:
            print(f"plot.py: Warning - failed to load {acc_csv}: {e}", file=sys.stderr)
    return None


def plot_accuracy_platform_page(
    acc_df, platform, libs, tasks, page, n_pages, generated_at
):
    """Plots per-platform accuracy bar charts (Mean ULP) for each task, matching time bar chart styling."""
    n_rows, n_cols = GRID_SHAPE
    fig, axes = plt.subplots(n_rows, n_cols, figsize=(16, 8))
    axes = axes.flatten()

    plat_acc = acc_df[acc_df["platform"] == platform] if acc_df is not None else None

    has_any_plot = False
    x = list(range(len(libs)))

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
            lib in acc_indexed.index and pd.notna(acc_indexed.loc[lib, "mean_ulp"]) and not math.isinf(acc_indexed.loc[lib, "mean_ulp"])
            for lib in libs
        ]

        x_present = [xi for xi, p in zip(x, present) if p]
        if not x_present:
            ax.axis("off")
            continue

        has_any_plot = True
        ulps_present = [float(acc_indexed.loc[lib, "mean_ulp"]) for lib, p in zip(libs, present) if p]
        colors_present = [LIBRARY_COLORS.get(lib, FALLBACK_COLOR) for lib, p in zip(libs, present) if p]
        libs_present = [lib for lib, p in zip(libs, present) if p]

        ax.set_title(f"{task} Accuracy", fontsize=9, pad=14)

        # Plot bars: ULP + 1.0 on log scale so 0 ULP exact matches sit nicely at 1
        bar_heights = [u + 1.0 for u in ulps_present]

        bars = ax.bar(x_present, bar_heights, color=colors_present, width=0.6, zorder=2)
        for bar, lib in zip(bars, libs_present):
            if lib == "crazyflie-fw":
                bar.set_hatch(CF_HATCH)

        ax.set_yscale("log")

        # Ensure y-axis always starts at baseline 1.0 (10^0 = 0 ULP) for consistent visual comparison
        # and provide generous headroom for value labels above the highest bar
        max_y = max(bar_heights) if bar_heights else 1.0
        top_y = max(max_y * 4.0, 10.0)
        ax.set_ylim(1.0, top_y)

        # Label values above bars
        for xi, u in zip(x_present, ulps_present):
            y_pos = u + 1.0
            if u == 0.0:
                lbl = "0 (exact)"
            elif u < 10.0:
                lbl = f"{u:.1f} ULP"
            elif u < 1000.0:
                lbl = f"{u:.0f} ULP"
            else:
                lbl = f"{u:.1e} ULP"

            ax.text(
                xi, y_pos * 1.25, lbl,
                ha="center", va="bottom", fontsize=7.5, fontweight="bold",
                zorder=5, clip_on=True
            )

        ax.set_ylabel("Mean ULP + 1 (log)", fontsize=8, color=LOG_COLOR, fontweight="bold")

        # Format y-axis ticks strictly as 10^0, 10^1, 10^2, 10^3, ...
        import matplotlib.ticker as ticker
        ax.yaxis.set_major_locator(ticker.LogLocator(base=10.0))
        ax.yaxis.set_major_formatter(ticker.LogFormatterMathtext(base=10.0))

        ax.set_xlim(-0.5, len(libs) - 0.5)
        ax.set_xticks(x)
        tick_labels = ax.set_xticklabels(libs, rotation=30, ha="right", fontsize=8)
        for label, p in zip(tick_labels, present):
            if not p:
                label.set_color("#aaaaaa")
                label.set_style("italic")

        ax.grid(axis="y", color="#d0d0d0", linewidth=0.6, zorder=0)
        ax.set_axisbelow(True)

    if not has_any_plot:
        plt.close(fig)
        return None

    for ax in axes[len(tasks):]:
        ax.axis("off")

    title = f"{platform} Accuracy (Mean ULP Error)"
    title = title if n_pages == 1 else f"{title} (page {page}/{n_pages})"
    fig.suptitle(title, fontsize=18, fontweight="bold", y=0.99)
    fig.text(
        0.5, 0.94,
        "ULP (Unit in the Last Place): Float32 LSB bit distance. 0 ULP = Bit-exact match, 1-2 ULP = IEEE float noise",
        ha="center", fontsize=9, fontstyle="italic", color="#444444",
    )
    fig.text(
        0.99, 0.99,
        f"generated {generated_at}",
        ha="right", va="top", fontsize=9, color="#555555",
    )

    handles = [
        Patch(
            facecolor=LIBRARY_COLORS.get(lib, FALLBACK_COLOR),
            hatch=CF_HATCH if lib == "crazyflie-fw" else None,
            label=lib,
        )
        for lib in libs
    ]
    fig.legend(handles=handles, loc="lower center", ncol=len(libs), frameon=False)
    fig.subplots_adjust(left=0.055, right=0.985, top=0.86, bottom=0.13, hspace=0.7, wspace=0.35)
    return fig


def compute_pareto_frontier(points):
    """
    Given points = [(lib, runtime, error), ...], returns list of Pareto-optimal points
    sorted by runtime ascending. Lower runtime and lower error are preferred.
    """
    valid_points = [p for p in points if not pd.isna(p[1]) and not pd.isna(p[2])]
    if not valid_points:
        return []

    sorted_pts = sorted(valid_points, key=lambda p: (p[1], p[2]))
    pareto = []
    min_error = float('inf')

    for pt in sorted_pts:
        if pt[2] <= min_error:
            pareto.append(pt)
            min_error = pt[2]

    return pareto


def plot_pareto_summary_table_page(agg, acc_df, platform, tasks, generated_at):
    """Renders a clean Pareto Classification Summary Table for a platform at the end of the report."""
    plat_agg = agg[agg["platform"] == platform]
    plat_acc = acc_df[acc_df["platform"] == platform] if acc_df is not None else None
    if plat_agg.empty or plat_acc is None:
        return None

    table_data = []
    for task in tasks:
        sub_agg = plat_agg[plat_agg["task"] == task]
        sub_acc = plat_acc[plat_acc["task"] == task]
        merged = pd.merge(sub_agg, sub_acc, on=["platform", "library", "task"], how="inner")
        if merged.empty:
            continue

        points = []
        for _, row in merged.iterrows():
            lib = row["library"]
            runtime = row["median"]
            ulp = row["mean_ulp"]
            points.append((lib, runtime, ulp))

        if not points:
            continue

        fastest = min(points, key=lambda p: p[1])
        valid_acc = [p for p in points if not pd.isna(p[2]) and not math.isinf(p[2])]

        if valid_acc:
            min_ulp_val = min(p[2] for p in valid_acc)
            most_acc_libs = [p for p in valid_acc if abs(p[2] - min_ulp_val) < 0.1]
            names = "/".join(sorted([p[0] for p in most_acc_libs]))
            most_acc_str = f"{names} ({min_ulp_val:.1f} ULP)" if min_ulp_val < 1000.0 else f"{names} ({min_ulp_val:.1e} ULP)"
        else:
            most_acc_str = "N/A"

        pareto_pts = compute_pareto_frontier(points)
        pareto_libs = {p[0] for p in pareto_pts}
        all_libs = {p[0] for p in points}
        dominated_libs = sorted(list(all_libs - pareto_libs))

        fastest_str = f"{fastest[0]} ({fastest[1]:.1f} ns)"
        pareto_str = ", ".join(sorted(list(pareto_libs)))
        dominated_str = ", ".join(dominated_libs) if dominated_libs else "None"

        table_data.append([task, fastest_str, most_acc_str, pareto_str, dominated_str])

    if not table_data:
        return None

    fig, ax = plt.subplots(figsize=(14, 8))
    ax.axis("off")

    headers = ["Task", "Fastest Library", "Most Accurate Library", "Pareto Optimal Set", "Dominated Libraries"]
    table = ax.table(
        cellText=table_data,
        colLabels=headers,
        cellLoc="center",
        loc="center",
    )
    table.auto_set_font_size(False)
    table.set_fontsize(9)
    table.scale(1.2, 1.8)

    # Style table headers
    for i in range(len(headers)):
        table[(0, i)].set_facecolor("#2a78d6")
        table[(0, i)].set_text_props(color="white", fontweight="bold")

    fig.suptitle(f"{platform} - Speed & Accuracy Trade-off Summary", fontsize=16, fontweight="bold", y=0.96)
    fig.text(
        0.5, 0.90,
        "ULP (Unit in the Last Place): Measures float32 LSB bit distance (~1.19e-7 step at magnitude 1.0).\n0 ULP = Bit-exact float32 match | 1-2 ULP = Float noise | >10 ULP = Drift / Approximation.",
        ha="center", va="top", fontsize=9, style="italic", color="#444444"
    )
    fig.text(
        0.99, 0.99,
        f"generated {generated_at}",
        ha="right", va="top", fontsize=8, color="#555555",
    )
    return fig


if __name__ == "__main__":
    main(sys.argv)
