use crate::BenchmarkTask;
use crate::inputs::{MatMul3x3Input, RotateVectorInput};

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
