use mrs_benchmark_macros::benchmark_input;

#[benchmark_input("MatMul3x3")]
#[derive(Clone, Debug)]
pub struct MatMul3x3Input {
    pub lhs: [f32; 9],
    pub rhs: [f32; 9],
}

#[benchmark_input("RotateVector")]
#[derive(Clone, Debug)]
pub struct RotateVectorInput {
    pub point: [f32; 3],
    pub quat: [f32; 4], // [x, y, z, w]
}

#[benchmark_input("MatInverse3x3")]
#[derive(Clone, Debug)]
pub struct MatInverse3x3Input {
    pub matrix: [f32; 9],
}
