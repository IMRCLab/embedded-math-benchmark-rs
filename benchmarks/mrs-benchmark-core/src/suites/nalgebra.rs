use crate::TaskImplementation;
use crate::inputs::MatrixMul3x3Input;
use nalgebra::Matrix3;

pub struct NAlgMatMul3x3;

impl TaskImplementation<MatrixMul3x3Input, Matrix3<f32>> for NAlgMatMul3x3 {
    type PreparedInput = (Matrix3<f32>, Matrix3<f32>);
    type RawOutput = Matrix3<f32>;

    fn prepare(&self, input: &MatrixMul3x3Input) -> Self::PreparedInput {
        let lhs = Matrix3::from_row_slice(&input.lhs);
        let rhs = Matrix3::from_row_slice(&input.rhs);
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput {
        input.0 * input.1
    }

    fn finalize(&self, output: Self::RawOutput) -> Matrix3<f32> {
        output
    }
}
