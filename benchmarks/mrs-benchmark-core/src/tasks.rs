use crate::inputs::{
    Atan2Input, MatInverse3x3Input, MatMul3x3Input, RotateVectorInput, SinCosInput, SqrtInput,
};
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

pub struct Atan2;
impl BenchmarkTask for Atan2 {
    const IDENTIFIER: &'static str = "Atan2";
    type Input = Atan2Input;
    type Output = f32;
}

pub struct SinCos;
impl BenchmarkTask for SinCos {
    const IDENTIFIER: &'static str = "SinCos";
    type Input = SinCosInput;
    type Output = [f32; 2];
}

pub struct Sqrt;
impl BenchmarkTask for Sqrt {
    const IDENTIFIER: &'static str = "Sqrt";
    type Input = SqrtInput;
    type Output = f32;
}
