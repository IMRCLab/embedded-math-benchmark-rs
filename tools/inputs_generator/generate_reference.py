import argparse
import json
import os
import sys

from tasks import TASK_REGISTRY

def main():
    parser = argparse.ArgumentParser(description="Reference Ground Truth Generator using tasks.py")
    parser.add_argument("--inputs", type=str, default="../../benchmarks/inputs.json", help="Path to inputs.json")
    parser.add_argument("--output", type=str, default="../../benchmarks/reference_results.json", help="Path to output reference_results.json")
    args = parser.parse_args()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    inputs_path = args.inputs
    if inputs_path == "../../benchmarks/inputs.json":
        inputs_path = os.path.join(script_dir, inputs_path)

    out_path = args.output
    if out_path == "../../benchmarks/reference_results.json":
        out_path = os.path.join(script_dir, out_path)

    if not os.path.exists(inputs_path):
        print(f"Error: inputs file not found at {inputs_path}", file=sys.stderr)
        sys.exit(1)

    with open(inputs_path, "r") as f:
        config = json.load(f)

    reference_data = {"tasks": {}}

    cases = config.get("cases", [])
    for case in cases:
        task_name = case.get("test")
        if task_name not in TASK_REGISTRY:
            print(f"Warning: task '{task_name}' not found in TASK_REGISTRY. Skipping.", file=sys.stderr)
            continue

        task = TASK_REGISTRY[task_name]
        inputs_list = case.get("inputs", [])
        task_refs = []

        for idx, inp in enumerate(inputs_list):
            ref_res = task.compute_reference(inp)
            ref_entry = {
                "input_index": idx,
                **ref_res
            }
            task_refs.append(ref_entry)

        reference_data["tasks"][task_name] = task_refs
        print(f"Generated f64 reference for '{task_name}' ({len(task_refs)} inputs)")

    with open(out_path, "w") as f:
        json.dump(reference_data, f, indent=2)

    print(f"\nSuccessfully saved reference data to {out_path}")

if __name__ == "__main__":
    main()
