#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MatrixMul3x3Input {
    pub lhs: [f32; 9],
    pub rhs: [f32; 9],
}
