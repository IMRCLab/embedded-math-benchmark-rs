use mrs_benchmark_core::{
    inputs::{MatrixMul3x3Input, RotateVectorInput},
    suites::{
        glam::{GlamMatMul3x3, GlamRotateVector},
        micromath::{UMathMatMul3x3, UMathRotateVector},
        nalgebra::{NAlgMatMul3x3, NAlgRotateVector},
    },
    BenchmarkPlatform,
};
use std::time::Instant;

pub struct HostPlatform;

impl BenchmarkPlatform for HostPlatform {
    type Instant = Instant;

    fn now(&self) -> Self::Instant {
        Instant::now()
    }

    fn elapsed(&self, start: Self::Instant) -> u64 {
        start.elapsed().as_nanos() as u64
    }

    fn unit(&self) -> &'static str {
        "ns"
    }
}

fn main() {
    let mut platform = HostPlatform;
    platform.setup();

    let m1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let m2 = [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

    let inputs = [
        MatrixMul3x3Input { lhs: m1, rhs: m2 },
        MatrixMul3x3Input { lhs: m2, rhs: m1 },
    ];

    let total_nalg = platform.run_set(&NAlgMatMul3x3, &inputs);
    println!(
        "Host Benchmark Set: NAlgMatMul3x3 took {} {} total",
        total_nalg,
        platform.unit()
    );

    let total_glam = platform.run_set(&GlamMatMul3x3, &inputs);
    println!(
        "Host Benchmark Set: GlamMatMul3x3 took {} {} total",
        total_glam,
        platform.unit()
    );

    let total_umath = platform.run_set(&UMathMatMul3x3, &inputs);
    println!(
        "Host Benchmark Set: UMathMatMul3x3 took {} {} total",
        total_umath,
        platform.unit()
    );

    // Quaternion Rotation Benchmarks
    let rot_inputs = [
        RotateVectorInput {
            point: [1.0, 2.0, 3.0],
            // 90 degrees rotation around Z axis: sin(45) = cos(45) = 1/sqrt(2)
            // [x, y, z, w]
            quat: [
                0.0,
                0.0,
                core::f32::consts::FRAC_1_SQRT_2,
                core::f32::consts::FRAC_1_SQRT_2,
            ],
        },
        RotateVectorInput {
            point: [-1.5, 3.2, 0.0],
            // 45 degrees rotation around X axis: sin(22.5) = 0.3826834, cos(22.5) = 0.9238795
            // [x, y, z, w]
            quat: [0.3826834, 0.0, 0.0, 0.9238795],
        },
    ];

    let rot_nalg = platform.run_set(&NAlgRotateVector, &rot_inputs);
    println!(
        "Host Benchmark Set: NAlgRotateVector took {} {} total",
        rot_nalg,
        platform.unit()
    );

    let rot_glam = platform.run_set(&GlamRotateVector, &rot_inputs);
    println!(
        "Host Benchmark Set: GlamRotateVector took {} {} total",
        rot_glam,
        platform.unit()
    );

    let rot_umath = platform.run_set(&UMathRotateVector, &rot_inputs);
    println!(
        "Host Benchmark Set: UMathRotateVector took {} {} total",
        rot_umath,
        platform.unit()
    );
}
