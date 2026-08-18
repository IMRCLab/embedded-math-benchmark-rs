"""Time (ns) bar-chart report pages: one page per platform (Section 1) and the
by-task comparison across platforms (Section 2)."""

import random

import matplotlib.pyplot as plt
import pandas as pd

import common
from config import GRID_SHAPE, GROUP_GAP, LOG_COLOR, TASK_GRID_SHAPE, UNIT
from data import choose_scale


def _task_note(task_errors):
    """One short line naming libraries that had errored inputs for this task."""
    if not task_errors:
        return ""
    parts = [f"{lib}: {n} err" for lib, n in sorted(task_errors.items())]
    return ", ".join(parts)


def plot_platform_page(agg, df_ok, error_counts, platform, profile, libs, tasks, page, n_pages, generated_at, versions=None):
    n_rows, n_cols = GRID_SHAPE
    fig, axes = plt.subplots(n_rows, n_cols, figsize=(16, 8))
    axes = axes.flatten()

    platform_rows = agg[(agg["platform"] == platform) & (agg["profile"] == profile)]
    total_n = int(platform_rows["count"].sum())
    platform_samples = df_ok[(df_ok["platform"] == platform) & (df_ok["profile"] == profile)]
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
            lib: n for (p, prof, t, lib), n in error_counts.items()
            if p == platform and prof == profile and t == task
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

        x_present = [xi for xi, p in zip(x, present) if p]
        medians_present = [m for m, p in zip(medians, present) if p]
        libs_present = [lib for lib, p in zip(libs, present) if p]

        common.draw_bars(ax, x_present, medians_present, libs_present, width=0.6)

        # Jittered raw samples over each bar: shows the real spread/density
        # instead of a single min/max whisker that one freak sample can stretch.
        for xi, lib in zip(x, libs):
            vals = task_samples.loc[task_samples["library"] == lib, "per_ns"]
            common.scatter_jitter(ax, xi, vals.tolist(), rng)

        if scale == "log":
            ax.set_yscale("log")
            ax.set_ylabel(f"{UNIT} (log)", fontsize=8, color=LOG_COLOR, fontweight="bold")
        else:
            ax.set_ylabel(UNIT, fontsize=8)
        ax.margins(y=0.15)  # headroom for the value label above the tallest bar

        for xi, median in zip(x_present, medians_present):
            common.label_bar_value(ax, xi, median, scale)

        # Fixed x-extent regardless of how many bars actually drew, so every
        # task subplot on the page reads as part of the same set.
        ax.set_xlim(-0.5, len(libs) - 0.5)
        ax.set_xticks(x)
        tick_labels = ax.set_xticklabels(libs, rotation=30, ha="right", fontsize=8)
        common.style_absent_ticks(tick_labels, present)
        common.style_grid(ax)

    for ax in axes[len(tasks):]:
        ax.axis("off")

    title = f"{platform} [{profile}]" if n_pages == 1 else f"{platform} [{profile}] (page {page}/{n_pages})"
    common.page_header(fig, title, f"n={total_n} rows", generated_at)
    common.add_legend(fig, libs, versions)
    # Fixed margins, not tight_layout: tight_layout sizes cells from the tight
    # bbox of all axes including ones turned off, so a sparse continuation
    # page got extra padding and its one subplot visibly inflated.
    fig.subplots_adjust(left=0.055, right=0.985, top=0.89, bottom=0.13, hspace=0.7, wspace=0.35)
    return fig


def plot_task_page(agg, df_ok, error_counts, tasks, platform_profiles, libs, page, n_pages, generated_at, versions=None):
    """plot_platform_page transposed: one subplot per task, x-axis grouped by
    (platform, profile), bars within each group by library. Every group reserves the
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
        task_pp = [
            (p, prof) for (p, prof) in platform_profiles
            if not task_rows[(task_rows["platform"] == p) & (task_rows["profile"] == prof)].empty
        ]
        task_samples = df_ok[df_ok["task"] == task]

        task_n = int(task_rows["count"].max())
        ax.set_title(f"{task} (n={task_n})", fontsize=9, pad=14)

        # A per-(platform, library) breakdown (like plot_platform_page's note)
        # would run off the subplot at this bar count; total is enough here,
        # the per-platform pages already carry the itemized breakdown.
        task_error_total = sum(n for (p, prof, t, lib), n in error_counts.items() if t == task)
        if task_error_total:
            ax.text(
                0.5, 1.0, f"{task_error_total} errored input(s) across platforms",
                transform=ax.transAxes, ha="center", va="bottom",
                fontsize=6, color="#999999",
            )

        tick_positions = []
        tick_labels_text = []
        bar_x, bar_h, bar_libs = [], [], []

        for gi, (platform, profile) in enumerate(task_pp):
            base = gi * group_span
            tick_positions.append(base + (n_libs - 1) / 2)
            tick_labels_text.append(f"{platform}\n({profile})")

            plat_rows = (
                task_rows[(task_rows["platform"] == platform) & (task_rows["profile"] == profile)]
                .set_index("library")
                .reindex(libs)
            )
            plat_samples = task_samples[
                (task_samples["platform"] == platform) & (task_samples["profile"] == profile)
            ]

            for li, lib in enumerate(libs):
                x = base + li
                vals = plat_samples.loc[plat_samples["library"] == lib, "per_ns"]
                common.scatter_jitter(ax, x, vals.tolist(), rng)
                median = plat_rows.loc[lib, "median"]
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
        ax.margins(y=0.1)

        n_groups = max(len(task_pp), 1)
        ax.set_xlim(-0.5, (n_groups - 1) * group_span + (n_libs - 1) + 0.5)
        ax.set_xticks(tick_positions)
        ax.set_xticklabels(tick_labels_text, rotation=30, ha="right", fontsize=8)
        common.style_grid(ax)

    for ax in axes[len(tasks):]:
        ax.axis("off")

    title = "All platforms & profiles, grouped by task"
    title = title if n_pages == 1 else f"{title} (page {page}/{n_pages})"
    common.page_header(fig, title, f"n={total_n} rows", generated_at)
    common.add_legend(fig, libs, versions)
    fig.subplots_adjust(left=0.055, right=0.985, top=0.89, bottom=0.13, hspace=0.7, wspace=0.1)
    return fig
