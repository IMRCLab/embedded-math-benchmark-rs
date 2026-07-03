use crate::tasks::RotateVector as RotateVectorTask;
use crate::TaskImplementation;
use micromath::Quaternion;



pub struct RotateVector;

impl TaskImplementation<RotateVectorTask> for RotateVector {
    const LIBRARY_IDENTIFIER: &'static str = "micromath";

    type PreparedInput = (Quaternion, [f32; 3]);
    type RawOutput = Quaternion;

    fn prepare(&self, input: &<RotateVectorTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
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

    fn finalize(&self, output: Self::RawOutput) -> <RotateVectorTask as crate::BenchmarkTask>::Output {
        [output.x(), output.y(), output.z()]
    }
}
