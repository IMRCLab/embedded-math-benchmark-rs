use mrs_benchmark_macros::benchmark_input;

#[benchmark_input("MatMul3x3")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MatMul3x3Input {
    pub lhs: [f32; 9],
    pub rhs: [f32; 9],
}

#[benchmark_input("RotateVector")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct RotateVectorInput {
    pub point: [f32; 3],
    pub quat: [f32; 4], // [x, y, z, w]
}
