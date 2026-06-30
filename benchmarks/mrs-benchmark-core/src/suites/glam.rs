use crate::TaskImplementation;
use crate::inputs::MatrixMul3x3Input;
use glam::Mat3;

pub struct GlamMatMul3x3;

impl TaskImplementation<MatrixMul3x3Input, Mat3> for GlamMatMul3x3 {
    type PreparedInput = (Mat3, Mat3);
    type RawOutput = Mat3;

    fn prepare(&self, input: &MatrixMul3x3Input) -> Self::PreparedInput {
        let lhs = Mat3::from_cols_array(&input.lhs).transpose();
        let rhs = Mat3::from_cols_array(&input.rhs).transpose();
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput {
        input.0 * input.1
    }

    fn finalize(&self, output: Self::RawOutput) -> Mat3 {
        output
    }
}
