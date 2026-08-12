"""Shared chart chrome used by every report page: bar drawing/coloring, value-label
placement and styling, legend, grid/tick styling, jitter scatter, and page header/footer.
Kept separate from the page builders (pages_time.py/pages_accuracy.py) so each of those
only states what's actually different about its own chart."""

from matplotlib.patches import Patch

from config import FALLBACK_COLOR, LIBRARY_COLORS, LIBRARY_HATCHES

LOG_LABEL_MULTIPLIER = 1.18  # bar_top -> label y on a log axis
SYMLOG_LABEL_PIXEL_GAP = 10  # bar_top -> label y on a symlog axis, in display pixels


def paginate(items, page_size):
    return [items[i : i + page_size] for i in range(0, len(items), page_size)]


def scatter_jitter(ax, x_center, values, rng):
    """Jittered raw samples over a bar: shows the real spread/density instead of a
    single min/max whisker that one freak sample can stretch."""
    xs = [x_center + rng.uniform(-0.18, 0.18) for _ in range(len(values))]
    ax.scatter(xs, values, s=6, color="#0b0b0b", alpha=0.35, zorder=3, linewidths=0)


def format_compact(value):
    return f"{value / 1000:.2f}k" if value >= 1000 else f"{value:.0f}"


def bar_colors(libs):
    return [LIBRARY_COLORS.get(lib, FALLBACK_COLOR) for lib in libs]


def hatch_c_impl_bars(bars, libs):
    for bar, lib in zip(bars, libs):
        hatch = LIBRARY_HATCHES.get(lib)
        if hatch:
            bar.set_hatch(hatch)


def draw_bars(ax, x, heights, libs, *, bottom=0, width=0.6, zorder=2):
    bars = ax.bar(x, heights, color=bar_colors(libs), width=width, zorder=zorder, bottom=bottom)
    hatch_c_impl_bars(bars, libs)
    return bars


def label_offset(bar_top, scale, ax, linthresh=None):
    """Y position for a value label above bar_top. On a log axis, callers that set
    their own y-limit must reserve more headroom above the tallest bar than this
    multiplier uses (see pages_accuracy.py's MIN_HEADROOM_DECADES) -- otherwise a
    label can be sized for one limit and rendered past another."""
    if scale == "log":
        return bar_top * LOG_LABEL_MULTIPLIER
    if scale == "symlog":
        # A flat data-value offset lands very differently depending on where
        # bar_top falls: symlog's linear region (near zero) and log region
        # (above linthresh) have very different data-to-pixel density, so the
        # same offset can hug a bar in one chart and float well off it in
        # another (e.g. a chart whose bars are all sub-linthresh, where the
        # linear region fills most of the plot). Doing the offset in display
        # pixels instead keeps the visual gap constant everywhere.
        _, y_px = ax.transData.transform((0, bar_top))
        _, y_data = ax.transData.inverted().transform((0, y_px + SYMLOG_LABEL_PIXEL_GAP))
        return y_data
    lo, hi = ax.get_ylim()
    return bar_top + 0.045 * (hi - lo)


def draw_value_label(ax, x, y, text, rotation=0):
    ax.text(
        x, y, text, rotation=rotation,
        ha="center", va="bottom", fontsize=6.5, color="#222222", zorder=5,
        bbox=dict(boxstyle="round,pad=0.05", facecolor="white", edgecolor="none", alpha=0.45),
    )


def label_bar_value(ax, x_center, bar_top, scale, rotation=0):
    """Format+place+draw a compact numeric label above a bar (time-chart values)."""
    draw_value_label(ax, x_center, label_offset(bar_top, scale, ax), format_compact(bar_top), rotation)


def style_grid(ax):
    ax.grid(axis="y", color="#d0d0d0", linewidth=0.6, zorder=0)
    ax.set_axisbelow(True)


def style_absent_ticks(tick_labels, present):
    """Grey+italic the tick label for any slot a task/platform didn't implement."""
    for label, p in zip(tick_labels, present):
        if not p:
            label.set_color("#aaaaaa")
            label.set_style("italic")


def _legend_label(lib, versions):
    version = versions.get(lib)
    if not version:
        return lib
    return f"{lib} @{version}" if lib == "crazyflie-fw" else f"{lib} {version}"


def legend_handles(libs, versions=None):
    versions = versions or {}
    return [
        Patch(
            facecolor=LIBRARY_COLORS.get(lib, FALLBACK_COLOR),
            hatch=LIBRARY_HATCHES.get(lib),
            label=_legend_label(lib, versions),
        )
        for lib in libs
    ]


def add_legend(fig, libs, versions=None):
    fig.legend(handles=legend_handles(libs, versions), loc="lower center", ncol=len(libs), frameon=False)


def page_header(fig, title, subtitle, generated_at):
    fig.suptitle(title, fontsize=18, fontweight="bold", y=0.99)
    fig.text(0.5, 0.94, subtitle, ha="center", fontsize=9, fontstyle="italic", color="#444444")
    fig.text(0.99, 0.99, f"generated {generated_at}", ha="right", va="top", fontsize=9, color="#555555")
