use crate::TaskImplementation;
use crate::inputs::{MatrixMul3x3Input, RotateVectorInput};

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

pub struct NAlgRotateVector;

impl TaskImplementation<RotateVectorInput, [f32; 3]> for NAlgRotateVector {
    type PreparedInput = (nalgebra::UnitQuaternion<f32>, nalgebra::Vector3<f32>);
    type RawOutput = nalgebra::Vector3<f32>;

    fn prepare(&self, input: &RotateVectorInput) -> Self::PreparedInput {
        // nalgebra Quaternion::new expects (w, x, y, z)
        let q = nalgebra::Quaternion::new(input.quat[3], input.quat[0], input.quat[1], input.quat[2]);
        let uq = nalgebra::UnitQuaternion::from_quaternion(q);
        let v = nalgebra::Vector3::new(input.point[0], input.point[1], input.point[2]);
        (uq, v)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput {
        input.0 * input.1
    }

    fn finalize(&self, output: Self::RawOutput) -> [f32; 3] {
        [output.x, output.y, output.z]
    }
}

