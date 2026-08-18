"""Render results.csv into a per-platform multipage PDF report.

Usage: uv run --project viz viz/plot.py results.csv report.pdf
"""

import sys
from datetime import datetime
from zoneinfo import ZoneInfo

import matplotlib

# Must be set before pyplot import: pins the backend instead of relying on
# auto-detection, which picks a GUI backend (slower, display-dependent) when
# a display happens to be available.
matplotlib.use("Agg")

import matplotlib.pyplot as plt
from matplotlib.backends.backend_pdf import PdfPages

import common
import toc
from config import GRID_SHAPE, LIBRARY_ORDER, PLATFORM_ORDER, PROFILE_ORDER, TASK_GRID_SHAPE, TASK_ORDER
from data import aggregate, drop_platform, load_accuracy_df, load_and_clean, load_library_versions, ordered
from pages_accuracy import plot_accuracy_platform_page, plot_pareto_summary_table_page
from pages_time import plot_platform_page, plot_task_page


def _save(pdf, entries, fig, section, platform=None, profile=None):
    """Writes fig as the next page and records its page number for toc.py's
    later TOC-page + bookmark-outline pass."""
    pdf.savefig(fig)
    plt.close(fig)
    entries.append({"page": pdf.get_pagecount(), "section": section, "platform": platform, "profile": profile})


def main(argv):
    csv_path, out_pdf = argv[1], argv[2]
    df_ok, error_counts = load_and_clean(csv_path)
    agg = aggregate(df_ok)
    tasks = ordered(df_ok["task"].unique(), TASK_ORDER, "task(s)")
    page_size = GRID_SHAPE[0] * GRID_SHAPE[1]
    task_pages = common.paginate(tasks, page_size)
    by_task_pages = common.paginate(tasks, TASK_GRID_SHAPE[0] * TASK_GRID_SHAPE[1])
    generated_at = datetime.now(ZoneInfo("Europe/Berlin")).strftime("%Y-%m-%d %H:%M:%S %Z")
    versions = load_library_versions(csv_path)

    all_platforms = ordered(agg["platform"].unique(), PLATFORM_ORDER, "platform(s)")
    all_profiles = ordered(agg["profile"].unique(), PROFILE_ORDER, "profile(s)")
    all_libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies) globally")

    # Present (platform, profile) pairs
    platform_profiles = [
        (p, prof)
        for p in all_platforms
        for prof in all_profiles
        if not agg[(agg["platform"] == p) & (agg["profile"] == prof)].empty
    ]

    # host is not a useful reference point for the by-task comparison -- drop it
    # from the x-axis grouping *and* its row/error counts, not just hide it.
    task_agg, task_df_ok, task_error_counts = drop_platform(agg, df_ok, error_counts, "host")
    task_platform_profiles = [(p, prof) for (p, prof) in platform_profiles if p != "host"]

    entries = []
    with PdfPages(out_pdf) as pdf:
        for platform, profile in platform_profiles:
            platform_libs = ordered(
                df_ok.loc[
                    (df_ok["platform"] == platform) & (df_ok["profile"] == profile), "library"
                ].unique(),
                LIBRARY_ORDER,
                f"library(ies) on {platform} [{profile}]",
            )
            for page_idx, task_page in enumerate(task_pages, start=1):
                page_rows = agg[
                    (agg["platform"] == platform)
                    & (agg["profile"] == profile)
                    & (agg["task"].isin(task_page))
                ]
                if page_rows.empty:
                    continue
                fig = plot_platform_page(
                    agg, df_ok, error_counts, platform, profile, platform_libs, task_page,
                    page_idx, len(task_pages), generated_at, versions,
                )
                _save(pdf, entries, fig, "time", platform, profile)

        # Section 2: Execution Time (by task, host excluded -- see task_agg/task_df_ok above)
        for page_idx, task_page in enumerate(by_task_pages, start=1):
            page_rows = task_agg[task_agg["task"].isin(task_page)]
            if page_rows.empty:
                continue
            fig = plot_task_page(
                task_agg, task_df_ok, task_error_counts, task_page, task_platform_profiles, all_libs,
                page_idx, len(by_task_pages), generated_at, versions,
            )
            _save(pdf, entries, fig, "by_task")

        # Section 3: Accuracy Bar Charts, each platform immediately followed by its
        # own Section 4 Pareto speed/accuracy summary table (rather than batching
        # all summary tables at the very end) so a reader never has to flip far
        # from a platform's charts to find that platform's own verdict table.
        acc_df, samples_map = load_accuracy_df(csv_path)
        if acc_df is not None:
            for platform, profile in platform_profiles:
                platform_libs = ordered(
                    df_ok.loc[
                        (df_ok["platform"] == platform) & (df_ok["profile"] == profile), "library"
                    ].unique(),
                    LIBRARY_ORDER,
                    f"library(ies) on {platform} [{profile}]",
                )
                for page_idx, task_page in enumerate(task_pages, start=1):
                    fig = plot_accuracy_platform_page(
                        acc_df, samples_map, platform, profile, platform_libs, task_page,
                        page_idx, len(task_pages), generated_at, versions,
                    )
                    if fig is not None:
                        _save(pdf, entries, fig, "accuracy", platform, profile)

                summary_fig = plot_pareto_summary_table_page(
                    agg, acc_df, platform, profile, tasks, generated_at
                )
                if summary_fig is not None:
                    _save(pdf, entries, summary_fig, "pareto", platform, profile)

    toc.finalize(out_pdf, entries, all_platforms, all_profiles, generated_at)


if __name__ == "__main__":
    main(sys.argv)
