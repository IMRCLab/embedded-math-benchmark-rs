"""Render results.csv into a per-platform multipage PDF report.

Usage: uv run --project viz viz/plot.py results.csv report.pdf
"""

import sys
from datetime import datetime

import matplotlib

# Must be set before pyplot import: pins the backend instead of relying on
# auto-detection, which picks a GUI backend (slower, display-dependent) when
# a display happens to be available.
matplotlib.use("Agg")

import matplotlib.pyplot as plt
from matplotlib.backends.backend_pdf import PdfPages

import common
from config import GRID_SHAPE, LIBRARY_ORDER, PLATFORM_ORDER, TASK_GRID_SHAPE, TASK_ORDER
from data import aggregate, drop_platform, load_accuracy_df, load_and_clean, load_library_versions, ordered
from pages_accuracy import plot_accuracy_platform_page, plot_pareto_summary_table_page
from pages_time import plot_platform_page, plot_task_page


def main(argv):
    csv_path, out_pdf = argv[1], argv[2]
    df_ok, error_counts = load_and_clean(csv_path)
    agg = aggregate(df_ok)
    tasks = ordered(df_ok["task"].unique(), TASK_ORDER, "task(s)")
    page_size = GRID_SHAPE[0] * GRID_SHAPE[1]
    task_pages = common.paginate(tasks, page_size)
    by_task_pages = common.paginate(tasks, TASK_GRID_SHAPE[0] * TASK_GRID_SHAPE[1])
    generated_at = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    versions = load_library_versions()

    all_platforms = ordered(agg["platform"].unique(), PLATFORM_ORDER, "platform(s)")
    all_libs = ordered(df_ok["library"].unique(), LIBRARY_ORDER, "library(ies) globally")

    # host is not a useful reference point for the by-task comparison -- drop it
    # from the x-axis grouping *and* its row/error counts, not just hide it.
    task_agg, task_df_ok, task_error_counts = drop_platform(agg, df_ok, error_counts, "host")
    task_platforms = [p for p in all_platforms if p != "host"]

    with PdfPages(out_pdf) as pdf:
        for platform in all_platforms:
            platform_libs = ordered(
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
                    page_idx, len(task_pages), generated_at, versions,
                )
                pdf.savefig(fig)
                plt.close(fig)

        # Section 2: Execution Time (by task, host excluded -- see task_agg/task_df_ok above)
        for page_idx, task_page in enumerate(by_task_pages, start=1):
            page_rows = task_agg[task_agg["task"].isin(task_page)]
            if page_rows.empty:
                continue
            fig = plot_task_page(
                task_agg, task_df_ok, task_error_counts, task_page, task_platforms, all_libs,
                page_idx, len(by_task_pages), generated_at, versions,
            )
            pdf.savefig(fig)
            plt.close(fig)

        # Section 3: Accuracy Bar Charts, each platform immediately followed by its
        # own Section 4 Pareto speed/accuracy summary table (rather than batching
        # all summary tables at the very end) so a reader never has to flip far
        # from a platform's charts to find that platform's own verdict table.
        acc_df, samples_map = load_accuracy_df(csv_path)
        if acc_df is not None:
            for platform in all_platforms:
                platform_libs = ordered(
                    df_ok.loc[df_ok["platform"] == platform, "library"].unique(),
                    LIBRARY_ORDER,
                    f"library(ies) on {platform}",
                )
                for page_idx, task_page in enumerate(task_pages, start=1):
                    fig = plot_accuracy_platform_page(
                        acc_df, samples_map, platform, platform_libs, task_page,
                        page_idx, len(task_pages), generated_at, versions,
                    )
                    if fig is not None:
                        pdf.savefig(fig)
                        plt.close(fig)

                summary_fig = plot_pareto_summary_table_page(agg, acc_df, platform, tasks, generated_at)
                if summary_fig is not None:
                    pdf.savefig(summary_fig)
                    plt.close(summary_fig)


if __name__ == "__main__":
    main(sys.argv)
