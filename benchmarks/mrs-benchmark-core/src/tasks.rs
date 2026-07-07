use crate::inputs::{MatInverse3x3Input, MatMul3x3Input, RotateVectorInput};
use crate::BenchmarkTask;

pub struct MatMul3x3;
impl BenchmarkTask for MatMul3x3 {
    const IDENTIFIER: &'static str = "MatMul3x3";
    type Input = MatMul3x3Input;
    type Output = [f32; 9];
}

pub struct RotateVector;
impl BenchmarkTask for RotateVector {
    const IDENTIFIER: &'static str = "RotateVector";
    type Input = RotateVectorInput;
    type Output = [f32; 3];
}

pub struct MatInverse3x3;
impl BenchmarkTask for MatInverse3x3 {
    const IDENTIFIER: &'static str = "MatInverse3x3";
    type Input = MatInverse3x3Input;
    type Output = [f32; 9];
}
