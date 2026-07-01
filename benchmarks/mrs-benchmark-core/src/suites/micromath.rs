use crate::TaskImplementation;
use crate::inputs::{MatrixMul3x3Input, RotateVectorInput};
use micromath::{F32, Quaternion};

pub struct UMathMatMul3x3;

impl TaskImplementation<MatrixMul3x3Input, [f32; 9]> for UMathMatMul3x3 {
    type PreparedInput = ([F32; 9], [F32; 9]);
    type RawOutput = [f32; 9];

    fn prepare(&self, input: &MatrixMul3x3Input) -> Self::PreparedInput {
        let mut lhs = [F32(0.0); 9];
        let mut rhs = [F32(0.0); 9];
        for i in 0..9 {
            lhs[i] = F32(input.lhs[i]);
            rhs[i] = F32(input.rhs[i]);
        }
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput {
        let a = &input.0;
        let b = &input.1;

        let r0 = a[0]*b[0] + a[1]*b[3] + a[2]*b[6];
        let r1 = a[0]*b[1] + a[1]*b[4] + a[2]*b[7];
        let r2 = a[0]*b[2] + a[1]*b[5] + a[2]*b[8];

        let r3 = a[3]*b[0] + a[4]*b[3] + a[5]*b[6];
        let r4 = a[3]*b[1] + a[4]*b[4] + a[5]*b[7];
        let r5 = a[3]*b[2] + a[4]*b[5] + a[5]*b[8];

        let r6 = a[6]*b[0] + a[7]*b[3] + a[8]*b[6];
        let r7 = a[6]*b[1] + a[7]*b[4] + a[8]*b[7];
        let r8 = a[6]*b[2] + a[7]*b[5] + a[8]*b[8];

        [r0.0, r1.0, r2.0, r3.0, r4.0, r5.0, r6.0, r7.0, r8.0]
    }

    fn finalize(&self, output: Self::RawOutput) -> [f32; 9] {
        output
    }
}

pub struct UMathRotateVector;

impl TaskImplementation<RotateVectorInput, [f32; 3]> for UMathRotateVector {
    type PreparedInput = (Quaternion, [f32; 3]);
    type RawOutput = Quaternion;

    fn prepare(&self, input: &RotateVectorInput) -> Self::PreparedInput {
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

    fn finalize(&self, output: Self::RawOutput) -> [f32; 3] {
        [output.x(), output.y(), output.z()]
    }
}

