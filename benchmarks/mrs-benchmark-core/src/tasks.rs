use crate::inputs::{
    Atan2Input, DotProduct64DInput, EkfStepInput, LeeControllerInput, MatInverse3x3Input,
    MatInverse9x9Input, MatMul3x3Input, MatMul9x9Input, QuatMulInput, QuatSlerpInput,
    RotateVectorInput, SinCosInput, SqrtInput, UnitQuatMulInput,
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

pub struct QuatMul;
impl BenchmarkTask for QuatMul {
    const IDENTIFIER: &'static str = "QuatMul";
    type Input = QuatMulInput;
    type Output = [f32; 4];
}

pub struct UnitQuatMul;
impl BenchmarkTask for UnitQuatMul {
    const IDENTIFIER: &'static str = "UnitQuatMul";
    type Input = UnitQuatMulInput;
    type Output = [f32; 4];
}

pub struct QuatSlerp;
impl BenchmarkTask for QuatSlerp {
    const IDENTIFIER: &'static str = "QuatSlerp";
    type Input = QuatSlerpInput;
    type Output = [f32; 4];
}

pub struct LeeController;
impl BenchmarkTask for LeeController {
    const IDENTIFIER: &'static str = "LeeController";
    type Input = LeeControllerInput;
    type Output = [f32; 4];
}

pub struct EkfStep;
impl BenchmarkTask for EkfStep {
    const IDENTIFIER: &'static str = "EkfStep";
    type Input = EkfStepInput;
    type Output = [f32; 10];
}

pub struct MatMul9x9;
impl BenchmarkTask for MatMul9x9 {
    const IDENTIFIER: &'static str = "MatMul9x9";
    type Input = MatMul9x9Input;
    type Output = [f32; 81];
}

pub struct MatInverse9x9;
impl BenchmarkTask for MatInverse9x9 {
    const IDENTIFIER: &'static str = "MatInverse9x9";
    type Input = MatInverse9x9Input;
    type Output = [f32; 81];
}

pub struct DotProduct64D;
impl BenchmarkTask for DotProduct64D {
    const IDENTIFIER: &'static str = "DotProduct64D";
    type Input = DotProduct64DInput;
    type Output = f32;
}
