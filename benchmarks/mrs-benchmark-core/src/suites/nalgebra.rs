use crate::tasks::{MatMul3x3 as MatMul3x3Task, RotateVector as RotateVectorTask};
use crate::{BenchmarkError, BenchmarkLibrary, RawTaskImplementation, export_tasks};

use nalgebra::Matrix3;

pub struct Nalgebra;
impl BenchmarkLibrary for Nalgebra {
    const IDENTIFIER: &'static str = "nalgebra";
}

pub struct MatMul3x3Logic;

impl RawTaskImplementation<MatMul3x3Task> for MatMul3x3Logic {
    type PreparedInput = (Matrix3<f32>, Matrix3<f32>);
    type RawOutput = Matrix3<f32>;

    fn prepare(&self, input: &<MatMul3x3Task as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let lhs = Matrix3::from_row_slice(&input.lhs);
        let rhs = Matrix3::from_row_slice(&input.rhs);
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(&self, output: Self::RawOutput) -> Result<<MatMul3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        let mut arr = [0.0; 9];
        arr.copy_from_slice(output.as_slice());
        Ok(arr)
    }
}

pub struct RotateVectorLogic;

impl RawTaskImplementation<RotateVectorTask> for RotateVectorLogic {
    type PreparedInput = (nalgebra::UnitQuaternion<f32>, nalgebra::Vector3<f32>);
    type RawOutput = nalgebra::Vector3<f32>;

    fn prepare(&self, input: &<RotateVectorTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        // nalgebra Quaternion::new expects (w, x, y, z)
        let q = nalgebra::Quaternion::new(input.quat[3], input.quat[0], input.quat[1], input.quat[2]);
        let uq = nalgebra::UnitQuaternion::from_quaternion(q);
        let v = nalgebra::Vector3::new(input.point[0], input.point[1], input.point[2]);
        (uq, v)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(&self, output: Self::RawOutput) -> Result<<RotateVectorTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

export_tasks!(
    Nalgebra,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic
);
