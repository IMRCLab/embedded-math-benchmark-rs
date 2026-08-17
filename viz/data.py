"""Data loading and pure transforms: results.csv -> per-iteration ns, aggregated
medians, accuracy data, and the canonical-vs-present ordering used everywhere else."""

import json
import os
import sys

import pandas as pd

from config import LOG_THRESHOLD, PLATFORM_CLOCK_HZ


def ordered(present, canonical_order, what):
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
    (platform, profile, task, library) -> number of ERROR rows.
    """
    df = pd.read_csv(csv_path)
    if "profile" not in df.columns:
        df["profile"] = "release"

    res_str = df["result"].astype(str)
    duration_numeric = pd.to_numeric(df["duration"], errors="coerce")
    rep_numeric = pd.to_numeric(df["repetitions"], errors="coerce")

    is_error = (
        res_str.str.startswith("ERROR")
        | res_str.str.startswith("Err")
        | res_str.str.startswith("MathError")
        | duration_numeric.isna()
        | rep_numeric.isna()
        | (rep_numeric <= 0)
    )

    df_ok = df[~is_error].copy()
    df_ok["duration"] = duration_numeric[~is_error]
    df_ok["repetitions"] = rep_numeric[~is_error]
    df_ok["per"] = df_ok["duration"] / df_ok["repetitions"]
    df_ok["per_ns"] = to_nanoseconds(df_ok)

    error_counts = df[is_error].groupby(["platform", "profile", "task", "library"]).size().to_dict()
    return df_ok, error_counts


def aggregate(df_ok):
    """Per (platform, profile, task, library): median/count of `per_ns`."""
    return (
        df_ok.groupby(["platform", "profile", "task", "library"])["per_ns"]
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


def drop_platform(agg, df_ok, error_counts, platform):
    """Filtered copies of agg/df_ok/error_counts with `platform` removed entirely --
    used to keep a platform (e.g. host) out of both the x-axis and the row/error
    counts of a report section, not just hidden from view."""
    agg_f = agg[agg["platform"] != platform]
    df_f = df_ok[df_ok["platform"] != platform]
    errors_f = {k: v for k, v in error_counts.items() if k[0] != platform}
    return agg_f, df_f, errors_f


def load_accuracy_df(csv_path):
    """Loads accuracy_results.csv and accuracy_results.json if present alongside csv_path or in root."""
    csv_dir = os.path.dirname(os.path.abspath(csv_path))
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    acc_csv = os.path.join(csv_dir, "accuracy_results.csv")
    if not os.path.exists(acc_csv):
        acc_csv = os.path.join(root_dir, "accuracy_results.csv")

    acc_json = os.path.join(csv_dir, "accuracy_results.json")
    if not os.path.exists(acc_json):
        acc_json = os.path.join(root_dir, "accuracy_results.json")

    df = None
    samples_map = {}

    if os.path.exists(acc_csv):
        try:
            df = pd.read_csv(acc_csv)
            if "profile" not in df.columns:
                df["profile"] = "release"
        except Exception as e:
            print(f"plot.py: Warning - failed to load {acc_csv}: {e}", file=sys.stderr)

    if os.path.exists(acc_json):
        try:
            with open(acc_json) as f:
                data = json.load(f)
                for entry in data:
                    key = (
                        entry.get("platform"),
                        entry.get("profile", "release"),
                        entry.get("task"),
                        entry.get("library"),
                    )
                    if "ulp_samples" in entry:
                        samples_map[key] = entry["ulp_samples"]
        except Exception as e:
            print(f"plot.py: Warning - failed to load {acc_json}: {e}", file=sys.stderr)

    return df, samples_map


def load_library_versions(csv_path=None):
    """Loads library_versions.json (library -> version string) from csv_dir or workspace root."""
    paths_to_try = []
    if csv_path:
        paths_to_try.append(os.path.join(os.path.dirname(os.path.abspath(csv_path)), "library_versions.json"))
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    paths_to_try.append(os.path.join(root_dir, "library_versions.json"))

    for path in paths_to_try:
        if os.path.exists(path):
            try:
                with open(path) as f:
                    return json.load(f)
            except Exception as e:
                print(f"plot.py: Warning - failed to load {path}: {e}", file=sys.stderr)
    return {}


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
    min_error = float("inf")

    for pt in sorted_pts:
        if pt[2] <= min_error:
            pareto.append(pt)
            min_error = pt[2]

    return pareto
