import argparse
from abc import ABC, abstractmethod
import json
import os
import numpy as np

# Mapping of which libraries support which tests
TEST_LIBRARY_MAPPING = {
    "MatMul3x3": ["glam", "nalgebra", "crazyflie-fw"],
    "RotateVector": ["glam", "nalgebra", "micromath", "crazyflie-fw"],
    "MatInverse3x3": ["glam", "nalgebra"],
    "Atan2": ["libm", "micromath", "crazyflie-fw"],
    "SinCos": ["libm", "micromath", "crazyflie-fw"],
    "Sqrt": ["libm", "micromath", "crazyflie-fw"],
    "QuatMul": ["glam", "nalgebra", "micromath", "crazyflie-fw"],
    "QuatSlerp": ["glam", "nalgebra", "micromath", "crazyflie-fw"],
}

class TestCaseGenerator(ABC):
    def __init__(self, name: str, repetitions: int):
        self.name = name
        self.repetitions = repetitions
        self.libraries = TEST_LIBRARY_MAPPING[name]

    @abstractmethod
    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        """Generate test inputs (both standard and edge cases)."""
        pass

    def to_dict(self, rng: np.random.Generator) -> dict:
        return {
            "repetitions": self.repetitions,
            "libraries": self.libraries,
            "test": self.name,
            "inputs": self.generate_inputs(rng)
        }

class MatMul3x3Generator(TestCaseGenerator):
    def __init__(self):
        super().__init__("MatMul3x3", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        
        # 1. Standard scale (20 cases)
        for _ in range(20):
            inputs.append({
                "lhs": rng.uniform(-10.0, 10.0, 9).tolist(),
                "rhs": rng.uniform(-10.0, 10.0, 9).tolist()
            })

        # 2. Large scale log-uniform (15 cases, elements up to 10^12)
        for _ in range(15):
            scale_l = 10 ** rng.uniform(3.0, 12.0)
            scale_r = 10 ** rng.uniform(3.0, 12.0)
            inputs.append({
                "lhs": rng.uniform(-scale_l, scale_l, 9).tolist(),
                "rhs": rng.uniform(-scale_r, scale_r, 9).tolist()
            })

        # 3. Tiny scale log-uniform (15 cases, elements down to 10^-12)
        for _ in range(15):
            scale_l = 10 ** rng.uniform(-12.0, -3.0)
            scale_r = 10 ** rng.uniform(-12.0, -3.0)
            inputs.append({
                "lhs": rng.uniform(-scale_l, scale_l, 9).tolist(),
                "rhs": rng.uniform(-scale_r, scale_r, 9).tolist()
            })

        return inputs

class RotateVectorGenerator(TestCaseGenerator):
    def __init__(self):
        super().__init__("RotateVector", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []

        # Helper to generate random unit quaternion
        def gen_unit_quat():
            q = rng.normal(size=4)
            return (q / np.linalg.norm(q)).tolist()

        # 1. Standard unit rotations & points (20 cases)
        for _ in range(20):
            inputs.append({
                "point": rng.uniform(-10.0, 10.0, 3).tolist(),
                "quat": gen_unit_quat()
            })

        # 2. Large scale points (15 cases, up to 10^12)
        for _ in range(15):
            scale = 10 ** rng.uniform(3.0, 12.0)
            inputs.append({
                "point": rng.uniform(-scale, scale, 3).tolist(),
                "quat": gen_unit_quat()
            })

        # 3. Tiny scale points (15 cases, down to 10^-12)
        for _ in range(15):
            scale = 10 ** rng.uniform(-12.0, -3.0)
            inputs.append({
                "point": rng.uniform(-scale, scale, 3).tolist(),
                "quat": gen_unit_quat()
            })

        return inputs

class MatInverse3x3Generator(TestCaseGenerator):
    def __init__(self):
        super().__init__("MatInverse3x3", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []

        # Helper to generate invertible matrix at a scale
        def gen_invertible(scale, min_det):
            while True:
                m = rng.uniform(-scale, scale, (3, 3))
                det = np.linalg.det(m)
                if abs(det) > min_det:
                    return m.flatten().tolist()

        # 1. Standard scale (20 cases)
        for _ in range(20):
            inputs.append({"matrix": gen_invertible(10.0, 0.5)})

        # 2. Large scale (10 cases, elements up to 10^8)
        for _ in range(10):
            scale = 10 ** rng.uniform(2.0, 8.0)
            # Det scales as scale^3, so threshold scales accordingly
            inputs.append({"matrix": gen_invertible(scale, 0.1 * (scale ** 3))})

        # 3. Tiny scale (10 cases, elements down to 10^-8)
        for _ in range(10):
            scale = 10 ** rng.uniform(-8.0, -2.0)
            inputs.append({"matrix": gen_invertible(scale, 0.1 * (scale ** 3))})

        # 4. Ill-conditioned near-singular matrices (8 cases)
        # We perturb a singular matrix to test precision limits
        for _ in range(8):
            a = rng.uniform(-10.0, 10.0, 3)
            b = rng.uniform(-10.0, 10.0, 3)
            # row3 = linear combination of row1 and row2 + tiny noise
            row1 = a
            row2 = b
            coeff = rng.uniform(-2.0, 2.0)
            noise = rng.uniform(-1e-7, 1e-7, 3)
            row3 = coeff * row1 + noise
            m = np.vstack([row1, row2, row3])
            inputs.append({"matrix": m.flatten().tolist()})

        # 5. Exact Singular / Error matrices (2 cases)
        inputs.append({"matrix": [0.0] * 9})
        inputs.append({"matrix": [1.0, 2.0, 3.0, 2.0, 4.0, 6.0, 3.0, 6.0, 9.0]})

        return inputs

class Atan2Generator(TestCaseGenerator):
    def __init__(self):
        super().__init__("Atan2", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        # Generate 50 points evenly distributed over quadrants and log-scales
        angles = np.linspace(-np.pi, np.pi, 50)
        
        for theta in angles:
            # Vary radius on a log-scale from 10^-15 to 10^15
            r = 10 ** rng.uniform(-15.0, 15.0)
            y = r * np.sin(theta)
            x = r * np.cos(theta)
            inputs.append({"y": float(y), "x": float(x)})

        return inputs

class SinCosGenerator(TestCaseGenerator):
    def __init__(self):
        super().__init__("SinCos", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []

        # 1. Standard angles evenly spaced in [-PI, PI] (40 cases)
        angles = np.linspace(-np.pi, np.pi, 40)
        for theta in angles:
            inputs.append({"theta": float(theta)})

        # 2. Large scale angles (10 cases, tests period reduction correctness)
        # Log-uniform spacing from 10^2 to 10^12
        for _ in range(10):
            theta = 10 ** rng.uniform(2.0, 12.0)
            inputs.append({"theta": float(theta)})

        return inputs

class SqrtGenerator(TestCaseGenerator):
    def __init__(self):
        super().__init__("Sqrt", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []

        # 1. Standard positive values on a log-scale from 10^-15 to 10^15 (45 cases)
        scales = 10 ** np.linspace(-15.0, 15.0, 45)
        for val in scales:
            inputs.append({"value": float(val)})

        # 2. Exact zero boundary (1 case)
        inputs.append({"value": 0.0})

        # 3. Negative values for error path (4 cases)
        for _ in range(4):
            inputs.append({"value": float(rng.uniform(-100.0, -0.1))})

        return inputs

class QuatMulGenerator(TestCaseGenerator):
    def __init__(self):
        super().__init__("QuatMul", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []

        # Helper to generate unit quaternion
        def gen_unit_quat():
            q = rng.normal(size=4)
            return q / np.linalg.norm(q)

        # 1. Standard unit rotations (30 cases)
        for _ in range(30):
            inputs.append({
                "lhs": gen_unit_quat().tolist(),
                "rhs": gen_unit_quat().tolist()
            })

        # 2. Off-normalized scaled quaternions (20 cases, scales from 10^-6 to 10^6)
        # Tests how libraries handle non-unit quaternion multiplication
        for _ in range(20):
            scale_l = 10 ** rng.uniform(-6.0, 6.0)
            scale_r = 10 ** rng.uniform(-6.0, 6.0)
            inputs.append({
                "lhs": (gen_unit_quat() * scale_l).tolist(),
                "rhs": (gen_unit_quat() * scale_r).tolist()
            })

        return inputs

class QuatSlerpGenerator(TestCaseGenerator):
    def __init__(self):
        super().__init__("QuatSlerp", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []

        def gen_unit_quat():
            q = rng.normal(size=4)
            return q / np.linalg.norm(q)

        # 1. Standard unit slerp with uniform t (35 cases)
        for _ in range(35):
            inputs.append({
                "from": gen_unit_quat().tolist(),
                "to": gen_unit_quat().tolist(),
                "t": float(rng.uniform(0.0, 1.0))
            })

        # 2. Opposite paths: dot close to -1.0 (5 cases)
        for _ in range(5):
            q1 = gen_unit_quat()
            # Negate and add tiny perturbation to make dot close to -1.0
            q2 = -q1 + rng.normal(scale=1e-5, size=4)
            q2 /= np.linalg.norm(q2)
            inputs.append({
                "from": q1.tolist(),
                "to": q2.tolist(),
                "t": float(rng.uniform(0.0, 1.0))
            })

        # 3. Near-identical paths: dot close to 1.0 (5 cases, tests linear fallback)
        for _ in range(5):
            q1 = gen_unit_quat()
            q2 = q1 + rng.normal(scale=1e-5, size=4)
            q2 /= np.linalg.norm(q2)
            inputs.append({
                "from": q1.tolist(),
                "to": q2.tolist(),
                "t": float(rng.uniform(0.0, 1.0))
            })

        # 4. Off-normalized slerp (5 cases)
        for _ in range(5):
            scale_f = 10 ** rng.uniform(-3.0, 3.0)
            scale_t = 10 ** rng.uniform(-3.0, 3.0)
            inputs.append({
                "from": (gen_unit_quat() * scale_f).tolist(),
                "to": (gen_unit_quat() * scale_t).tolist(),
                "t": float(rng.uniform(0.0, 1.0))
            })

        return inputs

def main():
    parser = argparse.ArgumentParser(description="OOP-Based Distributed Inputs Generator")
    parser.add_argument("--seed", type=int, default=42, help="Random seed for reproducibility")
    parser.add_argument("--output", type=str, default="../../benchmarks/inputs.json", help="Path to write inputs.json")
    args = parser.parse_args()

    # Create NumPy Generator
    rng = np.random.default_rng(args.seed)

    # Instantiate generators
    generators = [
        MatMul3x3Generator(),
        RotateVectorGenerator(),
        MatInverse3x3Generator(),
        Atan2Generator(),
        SinCosGenerator(),
        SqrtGenerator(),
        QuatMulGenerator(),
        QuatSlerpGenerator()
    ]

    # Run generation
    cases = [gen.to_dict(rng) for gen in generators]
    config = {"cases": cases}

    # Resolve output path
    out_path = args.output
    if out_path == "../../benchmarks/inputs.json":
        script_dir = os.path.dirname(os.path.abspath(__file__))
        out_path = os.path.join(script_dir, out_path)

    with open(out_path, "w") as f:
        json.dump(config, f, indent=2)
    print(f"Successfully generated 50 inputs/case with 100 repetitions in {out_path} (Seed: {args.seed})")

if __name__ == "__main__":
    main()
