# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "numpy",
#     "scipy",
# ]
# ///
import argparse
import json
import os
import numpy as np

from tasks import TASK_REGISTRY

def main():
    parser = argparse.ArgumentParser(description="Centralized Inputs Generator using tasks.py")
    parser.add_argument("--seed", type=int, default=42, help="Random seed for reproducibility")
    parser.add_argument("--output", type=str, default="../../benchmarks/inputs.json", help="Path to write inputs.json")
    args = parser.parse_args()

    rng = np.random.default_rng(args.seed)

    cases = []
    for task_name, task in TASK_REGISTRY.items():
        inputs = task.generate_inputs(rng)
        cases.append({
            "repetitions": task.repetitions,
            "libraries": task.libraries,
            "test": task.name,
            "inputs": inputs
        })

    config = {"cases": cases}

    out_path = args.output
    if out_path == "../../benchmarks/inputs.json":
        script_dir = os.path.dirname(os.path.abspath(__file__))
        out_path = os.path.join(script_dir, out_path)

    with open(out_path, "w") as f:
        json.dump(config, f, indent=2)
    print(f"Successfully generated inputs for {len(cases)} tasks in {out_path} (Seed: {args.seed})")

if __name__ == "__main__":
    main()
