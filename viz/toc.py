"""Front matter added after the report is fully rendered: a table-of-contents
page plus a nested PDF bookmark outline, built from the page number plot.py
recorded for every page it wrote. Kept separate so plot.py's render loop only
needs to record (page, section, platform, profile) tuples as it goes -- the
outline/link-annotation logic lives entirely here."""

import os

import matplotlib.pyplot as plt
from pypdf import PdfWriter
from pypdf.annotations import Link

import common

SECTION_TITLES = {
    "time": "Execution Time by Platform",
    "by_task": "Execution Time by Task",
    "accuracy": "Accuracy & Pareto",
}


def _first_page(entries, **match):
    """Lowest recorded page number among entries matching all given fields, or
    None. Page numbers are 1-based counts of pages appended to the PdfWriter
    so far, taken right after the matching page was appended."""
    matches = [e["page"] for e in entries if all(e.get(k) == v for k, v in match.items())]
    return min(matches) if matches else None


def _acc_or_pareto_page(entries, platform, profile):
    """The accuracy page's own first page, falling back to its Pareto summary
    table page for a (platform, profile) that has one but not the other."""
    return _first_page(entries, section="accuracy", platform=platform, profile=profile) or _first_page(
        entries, section="pareto", platform=platform, profile=profile
    )


def _toc_page(entries, all_platforms, all_profiles, generated_at):
    """Page numbers shown here are already +1: a TOC page is about to be
    inserted in front of everything, so this table stays correct even for a
    reader without a bookmark sidebar (print preview, some mobile viewers).

    Also returns a (rect_in_pdf_points, target_page_index) list so finalize()
    can lay a click-through link annotation over each cell -- target_page_index
    is un-incremented since it already lines up with the merged doc's 0-based
    page index once the TOC is inserted in front (see finalize())."""
    fig, ax = plt.subplots(figsize=(16, 8))
    ax.axis("off")
    fig.subplots_adjust(top=0.90, bottom=0.15, left=0.1, right=0.9)

    rows = []
    cell_targets = {}  # (row, col) in the rendered table -> target_page_index
    for platform in all_platforms:
        for profile in all_profiles:
            time_page = _first_page(entries, section="time", platform=platform, profile=profile)
            acc_page = _acc_or_pareto_page(entries, platform, profile)
            if time_page is None and acc_page is None:
                continue
            row = len(rows) + 1  # +1 for header row
            if time_page is not None:
                cell_targets[(row, 2)] = time_page
            if acc_page is not None:
                cell_targets[(row, 3)] = acc_page
            rows.append(
                [
                    platform,
                    profile,
                    f"Page {time_page + 1}" if time_page else "-",
                    f"Page {acc_page + 1}" if acc_page else "-",
                ]
            )

    headers = ["Platform", "Profile", "Execution Time", "Accuracy & Pareto"]
    table = ax.table(cellText=rows, colLabels=headers, cellLoc="center", loc="center", bbox=[0, 0, 1, 1])
    table.auto_set_font_size(False)
    table.set_fontsize(10)
    for i in range(len(headers)):
        table[(0, i)].set_facecolor("#2a78d6")
        table[(0, i)].set_text_props(color="white", fontweight="bold")
    for (row, col), target_page in cell_targets.items():
        table[(row, col)].set_text_props(color="#2a78d6", fontweight="bold")

    common.page_header(fig, "Table of Contents", "", generated_at)

    by_task_page = _first_page(entries, section="by_task")
    by_task_text = None
    if by_task_page is not None:
        by_task_text = fig.text(
            0.5,
            0.08,
            f"{SECTION_TITLES['by_task']}  ->  Page {by_task_page + 1}",
            ha="center",
            va="center",
            fontsize=11,
            fontweight="bold",
            color="#2a78d6",
            zorder=3,
        )

    # Cell (and by-task link) bboxes only settle once a renderer has actually laid out the page.
    fig.canvas.draw()
    renderer = fig.canvas.get_renderer()
    pts_per_px = 72.0 / fig.dpi
    links = []
    for (row, col), target_page in cell_targets.items():
        bbox = table[(row, col)].get_window_extent(renderer)
        rect = (bbox.x0 * pts_per_px, bbox.y0 * pts_per_px, bbox.x1 * pts_per_px, bbox.y1 * pts_per_px)
        links.append((rect, target_page))

    if by_task_text is not None:
        # Padded well past the visible text so the click target stays generous
        # without needing a visible button chrome around it.
        pad_px = 14
        bbox = by_task_text.get_window_extent(renderer)
        rect = (
            (bbox.x0 - pad_px) * pts_per_px,
            (bbox.y0 - pad_px) * pts_per_px,
            (bbox.x1 + pad_px) * pts_per_px,
            (bbox.y1 + pad_px) * pts_per_px,
        )
        links.append((rect, by_task_page))

    return fig, links


def _add_platform_profile_outline(writer, title, page_fn, all_platforms, all_profiles):
    """Adds a 3-level bookmark tree (title > platform > profile) from
    page_fn(platform, profile) -> page index or None, skipping combos with no
    page. Skips the whole section if page_fn never returns a page."""
    per_platform = {}
    for platform in all_platforms:
        for profile in all_profiles:
            page = page_fn(platform, profile)
            if page is not None:
                per_platform.setdefault(platform, []).append((profile, page))
    if not per_platform:
        return

    section_page = min(pages[0][1] for pages in per_platform.values())
    section_parent = writer.add_outline_item(title, section_page)
    for platform, profiles in per_platform.items():
        plat_parent = writer.add_outline_item(platform, profiles[0][1], parent=section_parent)
        for profile, page in profiles:
            writer.add_outline_item(profile, page, parent=plat_parent)


def finalize(out_pdf, entries, all_platforms, all_profiles, generated_at):
    """Inserts a TOC page at the front of out_pdf and adds the nested bookmark
    outline plus per-cell click-through links, then overwrites out_pdf in place."""
    toc_fig, toc_links = _toc_page(entries, all_platforms, all_profiles, generated_at)
    toc_path = out_pdf + ".toc.pdf"
    toc_fig.savefig(toc_path)
    plt.close(toc_fig)

    writer = PdfWriter()
    writer.append(toc_path)
    writer.append(out_pdf)
    os.remove(toc_path)

    # TOC lands at index 0, so an old 1-based page number `p` (recorded via
    # len(writer.pages) right after that page was appended) is exactly the
    # new 0-based index both the outline and these link annotations want.
    for rect, target_page in toc_links:
        link = writer.add_annotation(page_number=0, annotation=Link(rect=rect, target_page_index=target_page))
        # pypdf's Link leaves /Dest[0] as a bare page number, which most
        # readers won't resolve -- swap in a real indirect page reference,
        # matching what add_outline_item does for the bookmark tree below.
        link["/Dest"][0] = writer.pages[target_page].indirect_reference

    _add_platform_profile_outline(
        writer,
        SECTION_TITLES["time"],
        lambda platform, profile: _first_page(entries, section="time", platform=platform, profile=profile),
        all_platforms,
        all_profiles,
    )
    by_task_page = _first_page(entries, section="by_task")
    if by_task_page is not None:
        writer.add_outline_item(SECTION_TITLES["by_task"], by_task_page)
    _add_platform_profile_outline(
        writer,
        SECTION_TITLES["accuracy"],
        lambda platform, profile: _acc_or_pareto_page(entries, platform, profile),
        all_platforms,
        all_profiles,
    )

    tmp_path = out_pdf + ".tmp"
    with open(tmp_path, "wb") as f:
        writer.write(f)
    os.replace(tmp_path, out_pdf)
