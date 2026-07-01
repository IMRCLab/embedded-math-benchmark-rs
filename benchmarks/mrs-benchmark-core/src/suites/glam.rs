use crate::inputs::{MatrixMul3x3Input, RotateVectorInput};
use crate::TaskImplementation;

use glam::{Mat3, Quat, Vec3};

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

pub struct GlamRotateVector;

impl TaskImplementation<RotateVectorInput, [f32; 3]> for GlamRotateVector {
    type PreparedInput = (Quat, Vec3);
    type RawOutput = Vec3;

    fn prepare(&self, input: &RotateVectorInput) -> Self::PreparedInput {
        let q = Quat::from_xyzw(input.quat[0], input.quat[1], input.quat[2], input.quat[3]);
        let v = Vec3::new(input.point[0], input.point[1], input.point[2]);
        (q, v)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput {
        input.0 * input.1
    }

    fn finalize(&self, output: Self::RawOutput) -> [f32; 3] {
        [output.x, output.y, output.z]
    }
}
