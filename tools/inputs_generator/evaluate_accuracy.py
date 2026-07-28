import argparse
import csv
import json
import math
import os
import sys

from tasks import TASK_REGISTRY

def parse_result_field(raw_val: str):
    """Parses result string from CSV into float, list of floats, or error string."""
    raw_val = raw_val.strip()
    if not raw_val or raw_val.startswith("Err") or raw_val.startswith("MathError"):
        return raw_val

    # Remove outer quotes if present
    if raw_val.startswith('"') and raw_val.endswith('"'):
        raw_val = raw_val[1:-1].strip()

    # Bracketed array: e.g. [1.0, 2.0, 3.0] or [1.0 2.0 3.0]
    if raw_val.startswith("[") and raw_val.endswith("]"):
        inner = raw_val[1:-1].strip()
        if not inner:
            return []
        parts = inner.replace(",", " ").split()
        try:
            return [float(p) for p in parts]
        except ValueError:
            return raw_val

    try:
        return float(raw_val)
    except ValueError:
        return raw_val


def main():
    parser = argparse.ArgumentParser(description="Microbenchmark Accuracy Evaluator using tasks.py")
    parser.add_argument("--results", type=str, default="../../results.csv", help="Path to results.csv")
    parser.add_argument("--reference", type=str, default="../../benchmarks/reference_results.json", help="Path to reference_results.json")
    parser.add_argument("--output-json", type=str, default="../../accuracy_results.json", help="Path to write accuracy_results.json")
    parser.add_argument("--output-csv", type=str, default="../../accuracy_results.csv", help="Path to write accuracy_results.csv")
    args = parser.parse_args()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    results_path = args.results
    if results_path == "../../results.csv":
        results_path = os.path.join(script_dir, results_path)

    ref_path = args.reference
    if ref_path == "../../benchmarks/reference_results.json":
        ref_path = os.path.join(script_dir, ref_path)

    out_json_path = args.output_json
    if out_json_path == "../../accuracy_results.json":
        out_json_path = os.path.join(script_dir, out_json_path)

    out_csv_path = args.output_csv
    if out_csv_path == "../../accuracy_results.csv":
        out_csv_path = os.path.join(script_dir, out_csv_path)

    if not os.path.exists(results_path):
        print(f"Error: results file not found at {results_path}", file=sys.stderr)
        sys.exit(1)

    if not os.path.exists(ref_path):
        print(f"Error: reference file not found at {ref_path}", file=sys.stderr)
        sys.exit(1)

    with open(ref_path, "r") as f:
        ref_data = json.load(f).get("tasks", {})

    # Read results.csv
    # Row: platform,library,task,input_index,repetitions,duration,unit,result
    grouped = {}

    with open(results_path, "r") as f:
        reader = csv.DictReader(f)
        for row in reader:
            platform = row.get("platform", "unknown")
            library = row.get("library", "unknown")
            task_name = row.get("task", "unknown")
            input_idx = int(row.get("input_index", 0))
            raw_res = row.get("result", "")

            lib_output = parse_result_field(raw_res)

            key = (platform, library, task_name)
            if key not in grouped:
                grouped[key] = []
            grouped[key].append((input_idx, lib_output))

    summary_rows = []

    for (platform, library, task_name), run_list in grouped.items():
        if task_name not in TASK_REGISTRY:
            continue

        task = TASK_REGISTRY[task_name]
        task_refs = ref_data.get(task_name, [])
        ref_map = {r["input_index"]: r for r in task_refs}

        ulp_list = []
        rel_err_list = []
        abs_err_list = []
        exact_count = 0
        total_eval = 0
        error_count = 0

        for input_idx, lib_out in run_list:
            if input_idx not in ref_map:
                continue

            ref_entry = ref_map[input_idx]
            metrics = task.evaluate_accuracy(lib_out, ref_entry)

            total_eval += 1

            if "match_error" in metrics:
                if metrics["match_error"]:
                    exact_count += 1
                else:
                    error_count += 1
                continue

            ulp = metrics.get("ulp_max")
            if ulp is not None and not math.isnan(ulp):
                ulp_list.append(ulp)
                if ulp == 0.0:
                    exact_count += 1

            rel_err = metrics.get("rel_error")
            if rel_err is not None and not math.isnan(rel_err):
                rel_err_list.append(rel_err)

            abs_err = metrics.get("abs_error")
            if abs_err is not None and not math.isnan(abs_err):
                abs_err_list.append(abs_err)

        max_ulp = max(ulp_list) if ulp_list else float('nan')
        mean_ulp = (sum(ulp_list) / len(ulp_list)) if ulp_list else float('nan')
        max_rel_err = max(rel_err_list) if rel_err_list else float('nan')
        mean_rel_err = (sum(rel_err_list) / len(rel_err_list)) if rel_err_list else float('nan')
        mean_abs_err = (sum(abs_err_list) / len(abs_err_list)) if abs_err_list else float('nan')
        exact_pct = (exact_count / total_eval * 100.0) if total_eval > 0 else 0.0

        summary_rows.append({
            "platform": platform,
            "library": library,
            "task": task_name,
            "total_runs": total_eval,
            "max_ulp": max_ulp,
            "mean_ulp": mean_ulp,
            "max_rel_error": max_rel_err,
            "mean_rel_error": mean_rel_err,
            "mean_abs_error": mean_abs_err,
            "exact_bit_pct": exact_pct,
            "errors": error_count,
        })

    # Sort summary rows by task, platform, library
    summary_rows.sort(key=lambda x: (x["task"], x["platform"], x["library"]))

    # Write CSV
    if summary_rows:
        fieldnames = list(summary_rows[0].keys())
        with open(out_csv_path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(summary_rows)

    # Write JSON
    with open(out_json_path, "w") as f:
        json.dump(summary_rows, f, indent=2)

    # Print Markdown Summary
    print("\n# Microbenchmark Accuracy Evaluation Summary\n")
    print("| Task | Platform | Library | Max ULP | Mean ULP | Max Rel Error | Mean Rel Error | Bit-Exact (0 ULP) % | Errors |")
    print("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |")

    for r in summary_rows:
        max_u = f"{r['max_ulp']:.1f}" if not math.isnan(r['max_ulp']) else "N/A"
        mean_u = f"{r['mean_ulp']:.2f}" if not math.isnan(r['mean_ulp']) else "N/A"
        max_r = f"{r['max_rel_error']:.2e}" if not math.isnan(r['max_rel_error']) else "N/A"
        mean_r = f"{r['mean_rel_error']:.2e}" if not math.isnan(r['mean_rel_error']) else "N/A"
        exact_p = f"{r['exact_bit_pct']:.1f}%"

        print(f"| {r['task']} | {r['platform']} | {r['library']} | {max_u} | {mean_u} | {max_r} | {mean_r} | {exact_p} | {r['errors']} |")

    print(f"\nReport written to {out_csv_path} and {out_json_path}")

if __name__ == "__main__":
    main()
