use crate::tasks::RotateVector as RotateVectorTask;
use crate::{export_tasks, BenchmarkLibrary, RawTaskImplementation};
use micromath::Quaternion;

pub struct Micromath;
impl BenchmarkLibrary for Micromath {
    const IDENTIFIER: &'static str = "micromath";
}

pub struct RotateVectorLogic;

impl RawTaskImplementation<RotateVectorTask> for RotateVectorLogic {
    type PreparedInput = (Quaternion, [f32; 3]);
    type RawOutput = Quaternion;

    fn prepare(
        &self,
        input: &<RotateVectorTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        // micromath Quaternion::new takes (w, x, y, z)
        let q = Quaternion::new(input.quat[3], input.quat[0], input.quat[1], input.quat[2]);
        (q, input.point)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput {
        let q = &input.0;
        let p = &input.1;
        let q_vec = Quaternion::new(0.0, p[0], p[1], p[2]);
        *q * q_vec * q.conj()
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> <RotateVectorTask as crate::BenchmarkTask>::Output {
        [output.x(), output.y(), output.z()]
    }
}

export_tasks!(
    Micromath,
    RotateVector => RotateVectorLogic
);
