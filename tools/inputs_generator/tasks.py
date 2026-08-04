from __future__ import annotations
import math
from abc import ABC, abstractmethod
from typing import Any
import numpy as np

# ULP helper for single precision floats
def float32_to_bits(val: float) -> int:
    """Convert a 32-bit float to its IEEE 754 bit representation as an unsigned int."""
    f32_val = np.float32(val)
    if np.isnan(f32_val):
        return 0x7FC00000
    return int(f32_val.view(np.uint32))

def ulp_distance_f32(val1: float, val2: float) -> float:
    """
    Computes ULP (Units in the Last Place) distance between two floats in 32-bit IEEE 754 representation,
    normalized by the float32 LSB step size at scale max(|ref|, 1.0).
    This prevents artificial ULP explosions near function roots (zero-crossings) while strictly
    measuring float32 LSB precision everywhere.
    """
    f1 = float(val1)
    f2 = float(val2)
    if math.isnan(f1) or math.isnan(f2):
        return float('nan')
    if math.isinf(f1) or math.isinf(f2):
        return 0.0 if f1 == f2 else float('inf')
    if f1 == f2:
        return 0.0

    abs_err = abs(f1 - f2)
    scale = max(abs(f2), 1.0)
    spacing = float(np.spacing(np.float32(scale)))
    return float(abs_err / spacing)

def geodesic_quat_angle(q1: np.ndarray, q2: np.ndarray) -> float:
    """
    Geodesic rotation angle in radians between unit quaternions q1 and q2 ([x, y, z, w]).
    Handles q and -q symmetry.
    """
    dot = abs(np.dot(q1, q2))
    dot_clamped = min(1.0, max(0.0, dot))
    return 2.0 * math.acos(dot_clamped)

TEST_LIBRARY_MAPPING = {
    "MatMul3x3": ["glam", "nalgebra", "crazyflie-fw"],
    "RotateVector": ["glam", "nalgebra", "micromath", "crazyflie-fw"],
    "MatInverse3x3": ["glam", "nalgebra"],
    "Atan2": ["libm", "micromath", "crazyflie-fw"],
    "SinCos": ["libm", "micromath", "crazyflie-fw"],
    "Sqrt": ["libm", "micromath", "crazyflie-fw"],
    "QuatMul": ["glam", "micromath", "crazyflie-fw"],
    "UnitQuatMul": ["glam", "nalgebra", "micromath", "crazyflie-fw"],
    "QuatSlerp": ["glam", "nalgebra", "micromath", "crazyflie-fw"],
    "LeeController": ["glam", "nalgebra", "micromath", "crazyflie-fw"],
    "EkfStep": ["nalgebra", "crazyflie-fw", "micromath"],
    "MatMul9x9": ["nalgebra", "cmsis-dsp"],
    "MatInverse9x9": ["nalgebra", "cmsis-dsp"],
    "DotProduct64D": ["nalgebra", "cmsis-dsp"],
}

class BenchmarkTask(ABC):
    name: str
    repetitions: int
    libraries: list[str]
    platforms: list[str] | None

    def __init__(self, name: str, repetitions: int = 100, platforms: list[str] | None = None):
        self.name = name
        self.repetitions = repetitions
        self.libraries = TEST_LIBRARY_MAPPING[name]
        self.platforms = platforms

    @abstractmethod
    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        """Generate inputs for this task."""
        pass

    @abstractmethod
    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        """
        Compute high-precision f64 reference and f32 rounded reference.
        Returns dict with 'f64', 'f32', and optional 'error'.
        """
        pass

    def evaluate_accuracy(self, lib_output: Any, ref: dict[str, Any]) -> dict[str, Any]:
        """
        Calculates ULP distance, relative error, absolute error, and task-specific metrics.
        Default implementation handles scalar floats and 1D float arrays.
        """
        is_lib_err = lib_output is None or (isinstance(lib_output, str) and ("Error" in lib_output or "Err" in lib_output))
        is_ref_err = "error" in ref

        if is_ref_err or is_lib_err:
            match_err = (is_ref_err == is_lib_err)
            return {"match_error": match_err, "ulp_max": 0.0 if match_err else float('nan')}

        ref_f64 = np.array(ref["f64"], dtype=np.float64)
        ref_f32 = np.array(ref["f32"], dtype=np.float32)
        out_arr = np.array(lib_output, dtype=np.float32)

        if ref_f64.ndim == 0:  # Scalar
            val_out = float(out_arr)
            val_ref_f64 = float(ref_f64)
            val_ref_f32 = float(ref_f32)
            abs_err = abs(val_out - val_ref_f64)
            rel_err = abs_err / (abs(val_ref_f64) + 1e-12)
            ulp_dist = ulp_distance_f32(val_out, val_ref_f32)
            return {
                "abs_error": abs_err,
                "rel_error": rel_err,
                "ulp_max": ulp_dist,
                "ulp_mean": ulp_dist,
                "is_exact": (ulp_dist == 0.0),
            }
        else:  # Array / Vector
            flat_out = out_arr.flatten()
            flat_ref_f64 = ref_f64.flatten()
            flat_ref_f32 = ref_f32.flatten()

            abs_err_vec = np.abs(flat_out - flat_ref_f64)
            norm_ref = np.linalg.norm(flat_ref_f64)
            norm_diff = np.linalg.norm(flat_out - flat_ref_f64)
            rel_err = float(norm_diff / (norm_ref + 1e-12))
            abs_err_max = float(np.max(abs_err_vec))

            ulps = [ulp_distance_f32(o, r) for o, r in zip(flat_out, flat_ref_f32)]
            ulp_max = float(np.max(ulps))
            ulp_mean = float(np.mean(ulps))

            return {
                "abs_error": abs_err_max,
                "rel_error": rel_err,
                "ulp_max": ulp_max,
                "ulp_mean": ulp_mean,
                "is_exact": (ulp_max == 0.0),
            }

# --- Concrete Task Implementations ---

class MatMul3x3Task(BenchmarkTask):
    def __init__(self):
        super().__init__("MatMul3x3", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        for _ in range(20):
            inputs.append({
                "lhs": rng.uniform(-10.0, 10.0, 9).tolist(),
                "rhs": rng.uniform(-10.0, 10.0, 9).tolist()
            })
        for _ in range(15):
            sl = 10 ** rng.uniform(3.0, 12.0)
            sr = 10 ** rng.uniform(3.0, 12.0)
            inputs.append({
                "lhs": rng.uniform(-sl, sl, 9).tolist(),
                "rhs": rng.uniform(-sr, sr, 9).tolist()
            })
        for _ in range(15):
            sl = 10 ** rng.uniform(-12.0, -3.0)
            sr = 10 ** rng.uniform(-12.0, -3.0)
            inputs.append({
                "lhs": rng.uniform(-sl, sl, 9).tolist(),
                "rhs": rng.uniform(-sr, sr, 9).tolist()
            })
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        lhs = np.array(input_dict["lhs"], dtype=np.float64).reshape(3, 3)
        rhs = np.array(input_dict["rhs"], dtype=np.float64).reshape(3, 3)
        res = lhs @ rhs
        col_major_f64 = res.T.flatten().tolist()
        col_major_f32 = np.float32(col_major_f64).tolist()
        return {"f64": col_major_f64, "f32": col_major_f32}


class RotateVectorTask(BenchmarkTask):
    def __init__(self):
        super().__init__("RotateVector", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        def gen_unit_quat():
            q = rng.normal(size=4)
            return (q / np.linalg.norm(q)).tolist()

        for _ in range(20):
            inputs.append({"point": rng.uniform(-10.0, 10.0, 3).tolist(), "quat": gen_unit_quat()})
        for _ in range(15):
            scale = 10 ** rng.uniform(3.0, 12.0)
            inputs.append({"point": rng.uniform(-scale, scale, 3).tolist(), "quat": gen_unit_quat()})
        for _ in range(15):
            scale = 10 ** rng.uniform(-12.0, -3.0)
            inputs.append({"point": rng.uniform(-scale, scale, 3).tolist(), "quat": gen_unit_quat()})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        p = np.array(input_dict["point"], dtype=np.float64)
        q = np.array(input_dict["quat"], dtype=np.float64)
        qx, qy, qz, qw = q[0], q[1], q[2], q[3]
        qv = np.array([qx, qy, qz], dtype=np.float64)

        cross1 = np.cross(qv, p)
        cross2 = np.cross(qv, cross1)
        p_rot = p + 2.0 * qw * cross1 + 2.0 * cross2

        f64_res = p_rot.tolist()
        f32_res = np.float32(f64_res).tolist()
        return {"f64": f64_res, "f32": f32_res}


class MatInverse3x3Task(BenchmarkTask):
    def __init__(self):
        super().__init__("MatInverse3x3", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        def gen_invertible(scale, min_det):
            while True:
                m = rng.uniform(-scale, scale, (3, 3))
                det = np.linalg.det(m)
                if abs(det) > min_det:
                    return m.flatten().tolist()

        for _ in range(20):
            inputs.append({"matrix": gen_invertible(10.0, 0.5)})
        for _ in range(10):
            scale = 10 ** rng.uniform(2.0, 8.0)
            inputs.append({"matrix": gen_invertible(scale, 0.1 * (scale ** 3))})
        for _ in range(10):
            scale = 10 ** rng.uniform(-8.0, -2.0)
            inputs.append({"matrix": gen_invertible(scale, 0.1 * (scale ** 3))})
        for _ in range(8):
            row1 = rng.uniform(-10.0, 10.0, 3)
            row2 = rng.uniform(-10.0, 10.0, 3)
            coeff = rng.uniform(-2.0, 2.0)
            noise = rng.uniform(-1e-7, 1e-7, 3)
            row3 = coeff * row1 + noise
            m = np.vstack([row1, row2, row3])
            inputs.append({"matrix": m.flatten().tolist()})

        inputs.append({"matrix": [0.0] * 9})
        inputs.append({"matrix": [1.0, 2.0, 3.0, 2.0, 4.0, 6.0, 3.0, 6.0, 9.0]})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        m = np.array(input_dict["matrix"], dtype=np.float64).reshape(3, 3)
        det = float(np.linalg.det(m))
        if abs(det) < 1e-6:
            return {"error": "Matrix not invertible"}

        inv = np.linalg.inv(m)
        f64_col_major = inv.T.flatten().tolist()
        f32_col_major = np.float32(f64_col_major).tolist()
        return {"f64": f64_col_major, "f32": f32_col_major}

    def evaluate_accuracy(self, lib_output: Any, ref: dict[str, Any]) -> dict[str, Any]:
        metrics = super().evaluate_accuracy(lib_output, ref)
        if "abs_error" in metrics:
            metrics["residual_norm"] = float(metrics["abs_error"])
        return metrics


class Atan2Task(BenchmarkTask):
    def __init__(self):
        super().__init__("Atan2", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        angles = np.linspace(-np.pi, np.pi, 50)
        for theta in angles:
            r = 10 ** rng.uniform(-15.0, 15.0)
            inputs.append({"y": float(r * np.sin(theta)), "x": float(r * np.cos(theta))})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        y = float(input_dict["y"])
        x = float(input_dict["x"])
        res_f64 = float(np.arctan2(y, x))
        res_f32 = float(np.float32(res_f64))
        return {"f64": res_f64, "f32": res_f32}


class SinCosTask(BenchmarkTask):
    def __init__(self):
        super().__init__("SinCos", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        for theta in np.linspace(-np.pi, np.pi, 40):
            inputs.append({"theta": float(theta)})
        for _ in range(10):
            inputs.append({"theta": float(rng.uniform(-10.0 * np.pi, 10.0 * np.pi))})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        t = float(input_dict["theta"])
        f64_res = [float(np.sin(t)), float(np.cos(t))]
        f32_res = np.float32(f64_res).tolist()
        return {"f64": f64_res, "f32": f32_res}


class SqrtTask(BenchmarkTask):
    def __init__(self):
        super().__init__("Sqrt", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        for val in 10 ** np.linspace(-15.0, 15.0, 45):
            inputs.append({"value": float(val)})
        inputs.append({"value": 0.0})
        for _ in range(4):
            inputs.append({"value": float(rng.uniform(-100.0, -0.1))})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        val = float(input_dict["value"])
        if val < 0.0:
            return {"error": "Square root of negative number"}
        res_f64 = float(np.sqrt(val))
        res_f32 = float(np.float32(res_f64))
        return {"f64": res_f64, "f32": res_f32}


class QuatMulTask(BenchmarkTask):
    def __init__(self):
        super().__init__("QuatMul", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        def gen_unit():
            q = rng.normal(size=4)
            return q / np.linalg.norm(q)

        for _ in range(30):
            inputs.append({"lhs": gen_unit().tolist(), "rhs": gen_unit().tolist()})
        for _ in range(20):
            sl = 10 ** rng.uniform(-6.0, 6.0)
            sr = 10 ** rng.uniform(-6.0, 6.0)
            inputs.append({"lhs": (gen_unit() * sl).tolist(), "rhs": (gen_unit() * sr).tolist()})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        q1 = np.array(input_dict["lhs"], dtype=np.float64)
        q2 = np.array(input_dict["rhs"], dtype=np.float64)
        x1, y1, z1, w1 = q1[0], q1[1], q1[2], q1[3]
        x2, y2, z2, w2 = q2[0], q2[1], q2[2], q2[3]

        w = w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2
        x = w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2
        y = w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2
        z = w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2

        f64_res = [x, y, z, w]
        f32_res = np.float32(f64_res).tolist()
        return {"f64": f64_res, "f32": f32_res}

    def evaluate_accuracy(self, lib_output: Any, ref: dict[str, Any]) -> dict[str, Any]:
        metrics = super().evaluate_accuracy(lib_output, ref)
        if lib_output is not None:
            out_q = np.array(lib_output, dtype=np.float64)
            ref_q = np.array(ref["f64"], dtype=np.float64)
            n_out, n_ref = np.linalg.norm(out_q), np.linalg.norm(ref_q)
            if n_out > 1e-12 and n_ref > 1e-12:
                metrics["geodesic_angle_rad"] = geodesic_quat_angle(out_q / n_out, ref_q / n_ref)
        return metrics


class UnitQuatMulTask(BenchmarkTask):
    def __init__(self):
        super().__init__("UnitQuatMul", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        def gen_unit():
            q = rng.normal(size=4)
            return q / np.linalg.norm(q)

        for _ in range(50):
            inputs.append({"lhs": gen_unit().tolist(), "rhs": gen_unit().tolist()})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        q1 = np.array(input_dict["lhs"], dtype=np.float64)
        q2 = np.array(input_dict["rhs"], dtype=np.float64)
        q1 = q1 / np.linalg.norm(q1)
        q2 = q2 / np.linalg.norm(q2)

        x1, y1, z1, w1 = q1[0], q1[1], q1[2], q1[3]
        x2, y2, z2, w2 = q2[0], q2[1], q2[2], q2[3]

        w = w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2
        x = w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2
        y = w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2
        z = w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2

        res_q = np.array([x, y, z, w], dtype=np.float64)
        res_q /= np.linalg.norm(res_q)

        f64_res = res_q.tolist()
        f32_res = np.float32(f64_res).tolist()
        return {"f64": f64_res, "f32": f32_res}

    def evaluate_accuracy(self, lib_output: Any, ref: dict[str, Any]) -> dict[str, Any]:
        metrics = super().evaluate_accuracy(lib_output, ref)
        if lib_output is not None:
            out_q = np.array(lib_output, dtype=np.float64)
            ref_q = np.array(ref["f64"], dtype=np.float64)
            n_out, n_ref = np.linalg.norm(out_q), np.linalg.norm(ref_q)
            if n_out > 1e-12 and n_ref > 1e-12:
                metrics["geodesic_angle_rad"] = geodesic_quat_angle(out_q / n_out, ref_q / n_ref)
        return metrics


class QuatSlerpTask(BenchmarkTask):
    def __init__(self):
        super().__init__("QuatSlerp", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        def gen_unit():
            q = rng.normal(size=4)
            return q / np.linalg.norm(q)

        for _ in range(35):
            inputs.append({"from": gen_unit().tolist(), "to": gen_unit().tolist(), "t": float(rng.uniform(0.0, 1.0))})
        for _ in range(5):
            q1 = gen_unit()
            q2 = -q1 + rng.normal(scale=1e-5, size=4)
            q2 /= np.linalg.norm(q2)
            inputs.append({"from": q1.tolist(), "to": q2.tolist(), "t": float(rng.uniform(0.0, 1.0))})
        for _ in range(5):
            q1 = gen_unit()
            q2 = q1 + rng.normal(scale=1e-5, size=4)
            q2 /= np.linalg.norm(q2)
            inputs.append({"from": q1.tolist(), "to": q2.tolist(), "t": float(rng.uniform(0.0, 1.0))})
        for _ in range(5):
            q1 = gen_unit()
            q2 = gen_unit()
            inputs.append({"from": q1.tolist(), "to": q2.tolist(), "t": float(rng.choice([0.0, 1.0]))})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        q_from = np.array(input_dict["from"], dtype=np.float64)
        q_to = np.array(input_dict["to"], dtype=np.float64)
        t = float(input_dict["t"])

        q1 = q_from / np.linalg.norm(q_from)
        q2 = q_to / np.linalg.norm(q_to)

        dot = float(np.dot(q1, q2))
        if dot < 0.0:
            q2 = -q2
            dot = -dot

        if dot > 0.9995:
            res = (1.0 - t) * q1 + t * q2
            res /= np.linalg.norm(res)
        else:
            theta_0 = math.acos(dot)
            theta = theta_0 * t
            sin_theta_0 = math.sin(theta_0)
            s0 = math.cos(theta) - dot * math.sin(theta) / sin_theta_0
            s1 = math.sin(theta) / sin_theta_0
            res = s0 * q1 + s1 * q2

        f64_res = res.tolist()
        f32_res = np.float32(f64_res).tolist()
        return {"f64": f64_res, "f32": f32_res}

    def evaluate_accuracy(self, lib_output: Any, ref: dict[str, Any]) -> dict[str, Any]:
        metrics = super().evaluate_accuracy(lib_output, ref)
        if lib_output is not None:
            out_q = np.array(lib_output, dtype=np.float64)
            ref_q = np.array(ref["f64"], dtype=np.float64)
            n_out, n_ref = np.linalg.norm(out_q), np.linalg.norm(ref_q)
            if n_out > 1e-12 and n_ref > 1e-12:
                metrics["geodesic_angle_rad"] = geodesic_quat_angle(out_q / n_out, ref_q / n_ref)
        return metrics


class LeeControllerTask(BenchmarkTask):
    def __init__(self):
        super().__init__("LeeController", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        import csv, os
        inputs = []
        script_dir = os.path.dirname(os.path.abspath(__file__))
        csv_path = os.path.abspath(os.path.join(script_dir, "../assets/figure8_dt0_01.csv"))

        with open(csv_path, 'r') as f:
            rows = list(csv.DictReader(f))

        sampled = rng.choice(rows, 50, replace=False)
        for row in sampled:
            sp_pos = [float(row['posx']), float(row['posy']), float(row['posz'])]
            sp_vel = [float(row['velx']), float(row['vely']), float(row['velz'])]
            sp_acc = [float(row['accx']), float(row['accy']), float(row['accz'])]

            pos = [p + rng.uniform(-0.1, 0.1) for p in sp_pos]
            vel = [v + rng.uniform(-0.5, 0.5) for v in sp_vel]

            q = rng.normal(size=4)
            att = (q / np.linalg.norm(q)).tolist()
            ang_vel = rng.uniform(-2.0, 2.0, 3).tolist()

            inputs.append({
                "position": pos,
                "velocity": vel,
                "attitude": att,
                "angular_velocity": ang_vel,
                "setpoint_position": sp_pos,
                "setpoint_velocity": sp_vel,
                "setpoint_acceleration": sp_acc,
                "setpoint_yaw": 0.0,
                "setpoint_yaw_dot": 0.0,
                "mass": 0.033
            })
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        pos = np.array(input_dict["position"], dtype=np.float64)
        vel = np.array(input_dict["velocity"], dtype=np.float64)
        att = np.array(input_dict["attitude"], dtype=np.float64)
        ang_vel = np.array(input_dict["angular_velocity"], dtype=np.float64)

        cmd_pos = np.array(input_dict["setpoint_position"], dtype=np.float64)
        cmd_vel = np.array(input_dict["setpoint_velocity"], dtype=np.float64)
        cmd_acc = np.array(input_dict["setpoint_acceleration"], dtype=np.float64)
        cmd_yaw = float(input_dict["setpoint_yaw"])
        cmd_yaw_rate = float(input_dict["setpoint_yaw_dot"])
        mass = float(input_dict["mass"])

        KP = np.array([7.0, 7.0, 7.0], dtype=np.float64)
        KD = np.array([4.0, 4.0, 4.0], dtype=np.float64)
        K_R = np.array([0.007, 0.007, 0.008], dtype=np.float64)
        K_W = np.array([0.00115, 0.00115, 0.002], dtype=np.float64)
        GRAVITY = np.array([0.0, 0.0, -9.81], dtype=np.float64)  # matches crazyflie-fw's GRAVITY_MAGNITUDE
        INERTIA = np.array([1.657171e-5, 16.655602e-6, 29.261652e-6], dtype=np.float64)

        e_p = cmd_pos - pos
        e_v = cmd_vel - vel

        f_d = mass * (cmd_acc + KP * e_p + KD * e_v - GRAVITY)

        qx, qy, qz, qw = att[0], att[1], att[2], att[3]
        r_mat = np.array([
            [1.0 - 2.0*(qy**2 + qz**2), 2.0*(qx*qy - qz*qw),     2.0*(qx*qz + qy*qw)],
            [2.0*(qx*qy + qz*qw),     1.0 - 2.0*(qx**2 + qz**2), 2.0*(qy*qz - qx*qw)],
            [2.0*(qx*qz - qy*qw),     2.0*(qy*qz + qx*qw),     1.0 - 2.0*(qx**2 + qy**2)]
        ], dtype=np.float64)

        z_axis = np.array([0.0, 0.0, 1.0], dtype=np.float64)
        thrust = float(np.dot(f_d, r_mat @ z_axis))

        xcd = np.array([math.cos(cmd_yaw), math.sin(cmd_yaw), 0.0], dtype=np.float64)
        ycd = np.array([-math.sin(cmd_yaw), math.cos(cmd_yaw), 0.0], dtype=np.float64)

        cross_ycd_fd = np.cross(ycd, f_d)
        norm_cross1 = np.linalg.norm(cross_ycd_fd)
        xbd = cross_ycd_fd / norm_cross1 if norm_cross1 > 1e-12 else np.array([1.0, 0.0, 0.0])

        cross_fd_xbd = np.cross(f_d, xbd)
        norm_cross2 = np.linalg.norm(cross_fd_xbd)
        ybd = cross_fd_xbd / norm_cross2 if norm_cross2 > 1e-12 else np.array([0.0, 1.0, 0.0])
        zbd = np.cross(xbd, ybd)

        r_d = np.column_stack([xbd, ybd, zbd])

        r_d_t_r = r_d.T @ r_mat
        r_t_r_d = r_mat.T @ r_d
        err_mat = r_d_t_r - r_t_r_d
        e_r = 0.5 * np.array([err_mat[2, 1], err_mat[0, 2], err_mat[1, 0]], dtype=np.float64)

        f_d_len = np.linalg.norm(f_d)
        if f_d_len > 1e-7:
            cmd_jerk = np.zeros(3, dtype=np.float64)
            c = float(np.dot(zbd, cmd_acc - GRAVITY))
            d1 = float(np.dot(xbd, cmd_jerk))
            d2 = float(-np.dot(ybd, cmd_jerk))
            d3 = float(cmd_yaw_rate * np.dot(xcd, xbd))

            b3 = float(-np.dot(ycd, zbd))
            c3 = float(np.linalg.norm(np.cross(ycd, zbd)))

            wxd = d2 / c if abs(c) > 1e-12 else 0.0
            wyd = d1 / c if abs(c) > 1e-12 else 0.0
            wzd = (c * d3 - b3 * d1) / (c * c3) if abs(c * c3) > 1e-12 else 0.0
            w_d = np.array([wxd, wyd, wzd], dtype=np.float64)
        else:
            w_d = np.zeros(3, dtype=np.float64)

        e_w = ang_vel - r_mat.T @ r_d @ w_d
        feedback = -K_R * e_r - K_W * e_w
        gyro = np.cross(ang_vel, INERTIA * ang_vel)
        feed_forward = -INERTIA * np.cross(ang_vel, r_mat.T @ r_d @ w_d)
        torque = feedback + gyro + feed_forward

        f64_res = [thrust, torque[0], torque[1], torque[2]]
        f32_res = np.float32(f64_res).tolist()
        return {"f64": f64_res, "f32": f32_res}


class EkfStepTask(BenchmarkTask):
    def __init__(self):
        super().__init__("EkfStep", 100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        import csv, os
        inputs = []
        script_dir = os.path.dirname(os.path.abspath(__file__))
        csv_path = os.path.abspath(os.path.join(script_dir, "../assets/estimate.csv"))

        with open(csv_path, 'r') as f:
            rows = list(csv.DictReader(f))

        acc_rows = [r for r in rows if r.get('acc.x') and r.get('acc.x') != 'nan']
        gyro_rows = [r for r in rows if r.get('gyro.x') and r.get('gyro.x') != 'nan']
        pos_rows = [r for r in rows if r.get('stateEstimate.x') and r.get('stateEstimate.x') != 'nan']

        count = min(50, len(acc_rows), len(gyro_rows), len(pos_rows))
        acc_samples = rng.choice(acc_rows, count, replace=False)
        gyro_samples = rng.choice(gyro_rows, count, replace=False)
        pos_samples = rng.choice(pos_rows, count, replace=False)

        for i in range(count):
            r_acc = acc_samples[i]
            r_gyro = gyro_samples[i]
            r_pos = pos_samples[i]

            acc = [float(r_acc['acc.x']), float(r_acc['acc.y']), float(r_acc['acc.z'])]
            gyro = [float(r_gyro['gyro.x']), float(r_gyro['gyro.y']), float(r_gyro['gyro.z'])]

            pos = [float(r_pos['stateEstimate.x']), float(r_pos['stateEstimate.y']), float(r_pos['stateEstimate.z'])]
            v_world = np.array([float(r_pos['stateEstimate.vx']), float(r_pos['stateEstimate.vy']), float(r_pos['stateEstimate.vz'])], dtype=np.float64)

            qw = float(r_pos['stateEstimate.qw'])
            qx = float(r_pos['stateEstimate.qx'])
            qy = float(r_pos['stateEstimate.qy'])
            qz = float(r_pos['stateEstimate.qz'])

            q_norm = math.sqrt(qw*qw + qx*qx + qy*qy + qz*qz)
            if q_norm < 1e-6:
                att = [0.0, 0.0, 0.0, 1.0]
                qw, qx, qy, qz = 1.0, 0.0, 0.0, 0.0
            else:
                qw, qx, qy, qz = qw/q_norm, qx/q_norm, qy/q_norm, qz/q_norm
                att = [qx, qy, qz, qw]

            # Convert world velocity to body velocity: v_body = R(q)^T * v_world
            R_mat = np.array([
                [1.0 - 2.0*(qy**2 + qz**2), 2.0*(qx*qy - qz*qw),     2.0*(qx*qz + qy*qw)],
                [2.0*(qx*qy + qz*qw),     1.0 - 2.0*(qx**2 + qz**2), 2.0*(qy*qz - qx*qw)],
                [2.0*(qx*qz - qy*qw),     2.0*(qy*qz + qx*qw),     1.0 - 2.0*(qx**2 + qy**2)]
            ], dtype=np.float64)
            v_body = R_mat.T @ v_world

            zrange = float(r_pos['stateEstimate.z']) + float(rng.normal(0, 0.02))
            delta_n_x = float(rng.normal(0, 0.05))
            delta_n_y = float(rng.normal(0, 0.05))
            dt = 0.01

            cov = [0.0] * 81
            for k in range(3):
                cov[k*9 + k] = 0.1
                cov[(k+3)*9 + (k+3)] = 0.1
                cov[(k+6)*9 + (k+6)] = 0.01

            inputs.append({
                "position": pos,
                "velocity": v_body.tolist(),
                "attitude": att,
                "accelerometer": acc,
                "gyroscope": gyro,
                "range_z": zrange,
                "flow_delta": [delta_n_x, delta_n_y],
                "dt": dt,
                "covariance": cov
            })
        return inputs

    def compute_reference(self, input_dict: dict) -> dict[str, Any]:
        p = np.array(input_dict["position"], dtype=np.float64)
        v_b = np.array(input_dict["velocity"], dtype=np.float64)
        att = np.array(input_dict["attitude"], dtype=np.float64)  # [qx, qy, qz, qw]
        acc = np.array(input_dict["accelerometer"], dtype=np.float64)
        gyro = np.array(input_dict["gyroscope"], dtype=np.float64)
        zrange = float(input_dict["range_z"])
        dt = float(input_dict["dt"])
        cov = np.array(input_dict["covariance"], dtype=np.float64).reshape((9, 9))

    def compute_reference(self, input_dict: dict) -> dict:
        p = np.array(input_dict["position"], dtype=np.float64)
        v_b = np.array(input_dict["velocity"], dtype=np.float64)
        att = np.array(input_dict["attitude"], dtype=np.float64)
        acc = np.array(input_dict["accelerometer"], dtype=np.float64) * 9.81
        gyro = np.array(input_dict["gyroscope"], dtype=np.float64)
        zrange = float(input_dict["range_z"])
        dt = float(input_dict["dt"])
        cov = np.array(input_dict["covariance"], dtype=np.float64).reshape((9, 9))

        qx, qy, qz, qw = att[0], att[1], att[2], att[3]

        R = np.array([
            [1.0 - 2.0*(qy**2 + qz**2), 2.0*(qx*qy - qz*qw),     2.0*(qx*qz + qy*qw)],
            [2.0*(qx*qy + qz*qw),     1.0 - 2.0*(qx**2 + qz**2), 2.0*(qy*qz - qx*qw)],
            [2.0*(qx*qz - qy*qw),     2.0*(qy*qz + qx*qw),     1.0 - 2.0*(qx**2 + qy**2)]
        ], dtype=np.float64)

        dx = v_b[0] * dt
        dy = v_b[1] * dt
        dz = v_b[2] * dt + acc[2] * (dt**2) / 2.0

        p[0] += R[0, 0] * dx + R[0, 1] * dy + R[0, 2] * dz
        p[1] += R[1, 0] * dx + R[1, 1] * dy + R[1, 2] * dz
        p[2] += R[2, 0] * dx + R[2, 1] * dy + R[2, 2] * dz - 9.81 * (dt**2) / 2.0

        tmpSPX, tmpSPY, tmpSPZ = v_b[0], v_b[1], v_b[2]
        v_b[0] += dt * (gyro[2] * tmpSPY - gyro[1] * tmpSPZ - 9.81 * R[2, 0])
        v_b[1] += dt * (-gyro[2] * tmpSPX + gyro[0] * tmpSPZ - 9.81 * R[2, 1])
        v_b[2] += dt * (acc[2] + gyro[1] * tmpSPX - gyro[0] * tmpSPY - 9.81 * R[2, 2])

        dtwx, dtwy, dtwz = dt * gyro[0], dt * gyro[1], dt * gyro[2]
        angle = math.sqrt(dtwx**2 + dtwy**2 + dtwz**2)
        if angle > 1e-6:
            ca = math.cos(angle / 2.0)
            sa = math.sin(angle / 2.0)
            dq = np.array([sa * dtwx / angle, sa * dtwy / angle, sa * dtwz / angle, ca], dtype=np.float64)
            qw_n = qw*dq[3] - qx*dq[0] - qy*dq[1] - qz*dq[2]
            qx_n = qx*dq[3] + qw*dq[0] + qy*dq[2] - qz*dq[1]
            qy_n = qy*dq[3] + qw*dq[1] + qz*dq[0] - qx*dq[2]
            qz_n = qz*dq[3] + qw*dq[2] + qx*dq[1] - qy*dq[0]
            qx, qy, qz, qw = qx_n, qy_n, qz_n, qw_n

        A = np.eye(9, dtype=np.float64)
        A[0, 3] = R[0, 0] * dt; A[0, 4] = R[0, 1] * dt; A[0, 5] = R[0, 2] * dt
        A[1, 3] = R[1, 0] * dt; A[1, 4] = R[1, 1] * dt; A[1, 5] = R[1, 2] * dt
        A[2, 3] = R[2, 0] * dt; A[2, 4] = R[2, 1] * dt; A[2, 5] = R[2, 2] * dt

        A[3, 3] = 1.0;          A[3, 4] = gyro[2] * dt; A[3, 5] = -gyro[1] * dt
        A[4, 3] = -gyro[2] * dt; A[4, 4] = 1.0;          A[4, 5] = gyro[0] * dt
        A[5, 3] = gyro[1] * dt;  A[5, 4] = -gyro[0] * dt; A[5, 5] = 1.0

        A[3, 6] = 0.0;                   A[3, 7] = 9.81 * R[2, 2] * dt;  A[3, 8] = -9.81 * R[2, 1] * dt
        A[4, 6] = -9.81 * R[2, 2] * dt; A[4, 7] = 0.0;                   A[4, 8] = 9.81 * R[2, 0] * dt
        A[5, 6] = 9.81 * R[2, 1] * dt;  A[5, 7] = -9.81 * R[2, 0] * dt; A[5, 8] = 0.0

        d0, d1, d2 = gyro[0]*dt/2.0, gyro[1]*dt/2.0, gyro[2]*dt/2.0
        A[6, 6] = 1.0 - d1**2/2.0 - d2**2/2.0; A[6, 7] = d2 + d0*d1/2.0;            A[6, 8] = -d1 + d0*d2/2.0
        A[7, 6] = -d2 + d0*d1/2.0;            A[7, 7] = 1.0 - d0**2/2.0 - d2**2/2.0; A[7, 8] = d0 + d1*d2/2.0
        A[8, 6] = d1 + d0*d2/2.0;             A[8, 7] = -d0 + d1*d2/2.0;            A[8, 8] = 1.0 - d0**2/2.0 - d1**2/2.0

        R_proc = np.diag([0.0, 0.0, 0.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1])
        cov = A @ cov @ A.T + R_proc

        R = np.array([
            [1.0 - 2.0*(qy**2 + qz**2), 2.0*(qx*qy - qz*qw),     2.0*(qx*qz + qy*qw)],
            [2.0*(qx*qy + qz*qw),     1.0 - 2.0*(qx**2 + qz**2), 2.0*(qy*qz - qx*qw)],
            [2.0*(qx*qz - qy*qw),     2.0*(qy*qz + qx*qw),     1.0 - 2.0*(qx**2 + qy**2)]
        ], dtype=np.float64)

        expPointA, expStdA, expPointB, expStdB = 2.5, 0.0025, 4.0, 0.2
        expCoeff = math.log(expStdB / expStdA) / (expPointB - expPointA)
        stdDev = expStdA * (1.0 + math.exp(expCoeff * (p[2] - expPointA)))
        R_meas = stdDev * stdDev

        H = np.zeros((1, 9), dtype=np.float64)
        H[0, 2] = 1.0

        HT = H.T
        PHT = cov @ HT
        HPHR = R_meas + PHT[2, 0]
        K = PHT / HPHR
        error = zrange - p[2]

        S_state = np.array([p[0], p[1], p[2], v_b[0], v_b[1], v_b[2], 0.0, 0.0, 0.0], dtype=np.float64)
        S_state += K.flatten() * error
        p[0], p[1], p[2] = S_state[0], S_state[1], S_state[2]
        v_b[0], v_b[1], v_b[2] = S_state[3], S_state[4], S_state[5]
        d0, d1, d2 = S_state[6], S_state[7], S_state[8]

        I_KH = np.eye(9, dtype=np.float64) - K @ H
        cov = I_KH @ cov @ I_KH.T + K @ np.array([[R_meas]]) @ K.T

        dq_err = np.array([d0 / 2.0, d1 / 2.0, d2 / 2.0, 1.0], dtype=np.float64)
        qw_f = qw*dq_err[3] - qx*dq_err[0] - qy*dq_err[1] - qz*dq_err[2]
        qx_f = qx*dq_err[3] + qw*dq_err[0] + qy*dq_err[2] - qz*dq_err[1]
        qy_f = qy*dq_err[3] + qw*dq_err[1] + qz*dq_err[0] - qx*dq_err[2]
        qz_f = qz*dq_err[3] + qw*dq_err[2] + qx*dq_err[1] - qy*dq_err[0]
        q_final = np.array([qx_f, qy_f, qz_f, qw_f], dtype=np.float64)
        q_final = q_final / np.linalg.norm(q_final)

        res_f64 = np.concatenate([p, v_b, q_final]).tolist()
        res_f32 = np.float32(res_f64).tolist()
        return {"f64": res_f64, "f32": res_f32}


class MatMul9x9Task(BenchmarkTask):
    def __init__(self):
        super().__init__("MatMul9x9", repetitions=100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        for _ in range(50):
            lhs = rng.uniform(-5.0, 5.0, 81).astype(np.float32).tolist()
            rhs = rng.uniform(-5.0, 5.0, 81).astype(np.float32).tolist()
            inputs.append({"lhs": lhs, "rhs": rhs})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict:
        lhs = np.array(input_dict["lhs"], dtype=np.float64).reshape((9, 9))
        rhs = np.array(input_dict["rhs"], dtype=np.float64).reshape((9, 9))
        out = lhs @ rhs
        res_f64 = out.flatten().tolist()
        res_f32 = np.float32(res_f64).tolist()
        return {"f64": res_f64, "f32": res_f32}


class MatInverse9x9Task(BenchmarkTask):
    def __init__(self):
        super().__init__("MatInverse9x9", repetitions=100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        for _ in range(50):
            mat = rng.uniform(-2.0, 2.0, (9, 9)).astype(np.float32)
            mat = mat @ mat.T + np.eye(9, dtype=np.float32) * 5.0
            inputs.append({"matrix": mat.flatten().tolist()})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict:
        mat = np.array(input_dict["matrix"], dtype=np.float64).reshape((9, 9))
        out = np.linalg.inv(mat)
        res_f64 = out.flatten().tolist()
        res_f32 = np.float32(res_f64).tolist()
        return {"f64": res_f64, "f32": res_f32}


class DotProduct64DTask(BenchmarkTask):
    def __init__(self):
        super().__init__("DotProduct64D", repetitions=100)

    def generate_inputs(self, rng: np.random.Generator) -> list[dict]:
        inputs = []
        for _ in range(50):
            lhs = rng.uniform(-10.0, 10.0, 64).astype(np.float32).tolist()
            rhs = rng.uniform(-10.0, 10.0, 64).astype(np.float32).tolist()
            inputs.append({"lhs": lhs, "rhs": rhs})
        return inputs

    def compute_reference(self, input_dict: dict) -> dict:
        lhs = np.array(input_dict["lhs"], dtype=np.float64)
        rhs = np.array(input_dict["rhs"], dtype=np.float64)
        res_f64 = float(np.dot(lhs, rhs))
        res_f32 = float(np.float32(res_f64))
        return {"f64": res_f64, "f32": res_f32}


TASK_REGISTRY: dict[str, BenchmarkTask] = {
    "MatMul3x3": MatMul3x3Task(),
    "RotateVector": RotateVectorTask(),
    "MatInverse3x3": MatInverse3x3Task(),
    "Atan2": Atan2Task(),
    "SinCos": SinCosTask(),
    "Sqrt": SqrtTask(),
    "QuatMul": QuatMulTask(),
    "UnitQuatMul": UnitQuatMulTask(),
    "QuatSlerp": QuatSlerpTask(),
    "LeeController": LeeControllerTask(),
    "EkfStep": EkfStepTask(),
    "MatMul9x9": MatMul9x9Task(),
    "MatInverse9x9": MatInverse9x9Task(),
    "DotProduct64D": DotProduct64DTask(),
}
