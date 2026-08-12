use crate::tasks::{
    Atan2 as Atan2Task, CrossProduct as CrossProductTask, EkfStep as EkfStepTask, Exp as ExpTask,
    LeeController as LeeControllerTask, Ln as LnTask, MatMul3x3 as MatMul3x3Task,
    MatVecMul3x3 as MatVecMul3x3Task, QuatMul as QuatMulTask, QuatSlerp as QuatSlerpTask,
    QuatToRotMatrix as QuatToRotMatrixTask, RotateVector as RotateVectorTask, SinCos as SinCosTask,
    Sqrt as SqrtTask, UnitQuatMul as UnitQuatMulTask, Vec3Normalize as Vec3NormalizeTask,
};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};

pub struct CrazyflieFw;
impl BenchmarkLibrary for CrazyflieFw {
    const IDENTIFIER: &'static str = "crazyflie-fw";
}

pub struct MatMul3x3Logic;
impl RawTaskImplementation<MatMul3x3Task> for MatMul3x3Logic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::mat33,
        mrs_benchmark_crazyflie_sys::mat33,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::mat33;

    fn prepare(
        &self,
        input: &<MatMul3x3Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let lhs = mrs_benchmark_crazyflie_sys::mat33 {
            m: [
                [input.lhs[0], input.lhs[1], input.lhs[2]],
                [input.lhs[3], input.lhs[4], input.lhs[5]],
                [input.lhs[6], input.lhs[7], input.lhs[8]],
            ],
        };
        let rhs = mrs_benchmark_crazyflie_sys::mat33 {
            m: [
                [input.rhs[0], input.rhs[1], input.rhs[2]],
                [input.rhs[3], input.rhs[4], input.rhs[5]],
                [input.rhs[6], input.rhs[7], input.rhs[8]],
            ],
        };
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_mmul(input.0, input.1) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatMul3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([
            output.m[0][0],
            output.m[1][0],
            output.m[2][0],
            output.m[0][1],
            output.m[1][1],
            output.m[2][1],
            output.m[0][2],
            output.m[1][2],
            output.m[2][2],
        ])
    }
}

pub struct RotateVectorLogic;
impl RawTaskImplementation<RotateVectorTask> for RotateVectorLogic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::quat,
        mrs_benchmark_crazyflie_sys::vec,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::vec;

    fn prepare(
        &self,
        input: &<RotateVectorTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let q = mrs_benchmark_crazyflie_sys::quat {
            x: input.quat[0],
            y: input.quat[1],
            z: input.quat[2],
            w: input.quat[3],
        };
        let v = mrs_benchmark_crazyflie_sys::vec {
            x: input.point[0],
            y: input.point[1],
            z: input.point[2],
        };
        (q, v)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_qrot(input.0, input.1) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<RotateVectorTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

pub struct Atan2Logic;
impl RawTaskImplementation<Atan2Task> for Atan2Logic {
    type PreparedInput = (f32, f32);
    type RawOutput = f32;

    fn prepare(&self, input: &<Atan2Task as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        (input.y, input.x)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::atan2f(input.0, input.1) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<Atan2Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct SinCosLogic;
impl RawTaskImplementation<SinCosTask> for SinCosLogic {
    type PreparedInput = f32;
    type RawOutput = (f32, f32);

    fn prepare(&self, input: &<SinCosTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        input.theta
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        unsafe {
            let sin = mrs_benchmark_crazyflie_sys::sinf(*input);
            let cos = mrs_benchmark_crazyflie_sys::cosf(*input);
            Ok((sin, cos))
        }
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<SinCosTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.0, output.1])
    }
}

pub struct SqrtLogic;
impl RawTaskImplementation<SqrtTask> for SqrtLogic {
    type PreparedInput = f32;
    type RawOutput = f32;

    fn prepare(&self, input: &<SqrtTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        input.value
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        if *input < 0.0 {
            Err(BenchmarkError::MathError("Square root of negative number"))
        } else {
            Ok(unsafe { mrs_benchmark_crazyflie_sys::sqrtf(*input) })
        }
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<SqrtTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct QuatMulLogic;
impl RawTaskImplementation<QuatMulTask> for QuatMulLogic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::quat,
        mrs_benchmark_crazyflie_sys::quat,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::quat;

    fn prepare(&self, input: &<QuatMulTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let lhs = mrs_benchmark_crazyflie_sys::quat {
            x: input.lhs[0],
            y: input.lhs[1],
            z: input.lhs[2],
            w: input.lhs[3],
        };
        let rhs = mrs_benchmark_crazyflie_sys::quat {
            x: input.rhs[0],
            y: input.rhs[1],
            z: input.rhs[2],
            w: input.rhs[3],
        };
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_qqmul(input.1, input.0) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z, output.w])
    }
}

pub struct UnitQuatMulLogic;
impl RawTaskImplementation<UnitQuatMulTask> for UnitQuatMulLogic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::quat,
        mrs_benchmark_crazyflie_sys::quat,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::quat;

    fn prepare(
        &self,
        input: &<UnitQuatMulTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let q = |x: f32, y: f32, z: f32, w: f32| mrs_benchmark_crazyflie_sys::quat { x, y, z, w };
        // UnitQuatMulTask.generate_inputs always produces unit-norm quaternions, so
        // qnormalize's lack of a zero-magnitude guard is not a concern here.
        unsafe {
            (
                mrs_benchmark_crazyflie_sys::cf_qnormalize(q(
                    input.lhs[0],
                    input.lhs[1],
                    input.lhs[2],
                    input.lhs[3],
                )),
                mrs_benchmark_crazyflie_sys::cf_qnormalize(q(
                    input.rhs[0],
                    input.rhs[1],
                    input.rhs[2],
                    input.rhs[3],
                )),
            )
        }
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_qqmul(input.1, input.0) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<UnitQuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z, output.w])
    }
}

pub struct QuatSlerpLogic;
impl RawTaskImplementation<QuatSlerpTask> for QuatSlerpLogic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::quat,
        mrs_benchmark_crazyflie_sys::quat,
        f32,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::quat;

    fn prepare(
        &self,
        input: &<QuatSlerpTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let from = mrs_benchmark_crazyflie_sys::quat {
            x: input.from[0],
            y: input.from[1],
            z: input.from[2],
            w: input.from[3],
        };
        let to = mrs_benchmark_crazyflie_sys::quat {
            x: input.to[0],
            y: input.to[1],
            z: input.to[2],
            w: input.to[3],
        };
        (from, to, input.t)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_qslerp(input.0, input.1, input.2) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatSlerpTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z, output.w])
    }
}

pub struct EkfStepLogic;
impl RawTaskImplementation<EkfStepTask> for EkfStepLogic {
    type PreparedInput = <EkfStepTask as crate::BenchmarkTask>::Input;
    type RawOutput = [f32; 10];

    fn prepare(&self, input: &<EkfStepTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        input.clone()
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let mut out = [0.0f32; 10];
        unsafe {
            mrs_benchmark_crazyflie_sys::cf_ekf_step(
                input.position.as_ptr() as *mut f32,
                input.velocity.as_ptr() as *mut f32,
                input.attitude.as_ptr() as *mut f32,
                input.accelerometer.as_ptr(),
                input.gyroscope.as_ptr(),
                input.range_z,
                input.dt,
                input.covariance.as_ptr(),
                out.as_mut_ptr(),
            );
        }
        Ok(out)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<EkfStepTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct LeeControllerLogic;
impl RawTaskImplementation<LeeControllerTask> for LeeControllerLogic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::vec,
        mrs_benchmark_crazyflie_sys::vec,
        mrs_benchmark_crazyflie_sys::quat,
        mrs_benchmark_crazyflie_sys::vec,
        mrs_benchmark_crazyflie_sys::vec,
        mrs_benchmark_crazyflie_sys::vec,
        mrs_benchmark_crazyflie_sys::vec,
        f32,
        f32,
        f32,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::cf_lee_output;

    fn prepare(
        &self,
        input: &<LeeControllerTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let v = |a: [f32; 3]| mrs_benchmark_crazyflie_sys::vec {
            x: a[0],
            y: a[1],
            z: a[2],
        };
        (
            v(input.position),
            v(input.velocity),
            mrs_benchmark_crazyflie_sys::quat {
                x: input.attitude[0],
                y: input.attitude[1],
                z: input.attitude[2],
                w: input.attitude[3],
            },
            v(input.angular_velocity),
            v(input.setpoint_position),
            v(input.setpoint_velocity),
            v(input.setpoint_acceleration),
            input.setpoint_yaw,
            input.setpoint_yaw_dot,
            input.mass,
        )
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let (
            position,
            velocity,
            attitude,
            angular_velocity,
            setpoint_position,
            setpoint_velocity,
            setpoint_acceleration,
            setpoint_yaw,
            setpoint_yaw_dot,
            mass,
        ) = *input;
        Ok(unsafe {
            mrs_benchmark_crazyflie_sys::cf_lee_controller(
                position,
                velocity,
                attitude,
                angular_velocity,
                setpoint_position,
                setpoint_velocity,
                setpoint_acceleration,
                setpoint_yaw,
                setpoint_yaw_dot,
                mass,
            )
        })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<LeeControllerTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([
            output.thrust,
            output.torque.x,
            output.torque.y,
            output.torque.z,
        ])
    }
}

pub struct CrossProductLogic;
impl RawTaskImplementation<CrossProductTask> for CrossProductLogic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::vec,
        mrs_benchmark_crazyflie_sys::vec,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::vec;

    fn prepare(
        &self,
        input: &<CrossProductTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let v = |a: [f32; 3]| mrs_benchmark_crazyflie_sys::vec {
            x: a[0],
            y: a[1],
            z: a[2],
        };
        (v(input.lhs), v(input.rhs))
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_vcross(input.0, input.1) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<CrossProductTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

pub struct Vec3NormalizeLogic;
impl RawTaskImplementation<Vec3NormalizeTask> for Vec3NormalizeLogic {
    type PreparedInput = mrs_benchmark_crazyflie_sys::vec;
    type RawOutput = mrs_benchmark_crazyflie_sys::vec;

    fn prepare(
        &self,
        input: &<Vec3NormalizeTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        mrs_benchmark_crazyflie_sys::vec {
            x: input.vector[0],
            y: input.vector[1],
            z: input.vector[2],
        }
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_vnormalize(*input) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<Vec3NormalizeTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

pub struct MatVecMul3x3Logic;
impl RawTaskImplementation<MatVecMul3x3Task> for MatVecMul3x3Logic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::mat33,
        mrs_benchmark_crazyflie_sys::vec,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::vec;

    fn prepare(
        &self,
        input: &<MatVecMul3x3Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let m = mrs_benchmark_crazyflie_sys::mat33 {
            m: [
                [input.matrix[0], input.matrix[1], input.matrix[2]],
                [input.matrix[3], input.matrix[4], input.matrix[5]],
                [input.matrix[6], input.matrix[7], input.matrix[8]],
            ],
        };
        let v = mrs_benchmark_crazyflie_sys::vec {
            x: input.vector[0],
            y: input.vector[1],
            z: input.vector[2],
        };
        (m, v)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_mvmul(input.0, input.1) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatVecMul3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

pub struct QuatToRotMatrixLogic;
impl RawTaskImplementation<QuatToRotMatrixTask> for QuatToRotMatrixLogic {
    type PreparedInput = mrs_benchmark_crazyflie_sys::quat;
    type RawOutput = mrs_benchmark_crazyflie_sys::mat33;

    fn prepare(
        &self,
        input: &<QuatToRotMatrixTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let q = mrs_benchmark_crazyflie_sys::quat {
            x: input.quat[0],
            y: input.quat[1],
            z: input.quat[2],
            w: input.quat[3],
        };
        // Inputs are always near-unit quaternions (see QuatToRotMatrixTask.generate_inputs),
        // so qnormalize's lack of a zero-magnitude guard is not a concern here.
        unsafe { mrs_benchmark_crazyflie_sys::cf_qnormalize(q) }
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_quat2rotmat(*input) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatToRotMatrixTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([
            output.m[0][0],
            output.m[1][0],
            output.m[2][0],
            output.m[0][1],
            output.m[1][1],
            output.m[2][1],
            output.m[0][2],
            output.m[1][2],
            output.m[2][2],
        ])
    }
}

pub struct ExpLogic;
impl RawTaskImplementation<ExpTask> for ExpLogic {
    type PreparedInput = f32;
    type RawOutput = f32;

    fn prepare(&self, input: &<ExpTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        input.value
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::expf(*input) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<ExpTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct LnLogic;
impl RawTaskImplementation<LnTask> for LnLogic {
    type PreparedInput = f32;
    type RawOutput = f32;

    fn prepare(&self, input: &<LnTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        input.value
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        if *input <= 0.0 {
            Err(BenchmarkError::MathError(
                "Natural log of non-positive number",
            ))
        } else {
            Ok(unsafe { mrs_benchmark_crazyflie_sys::logf(*input) })
        }
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<LnTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

export_tasks!(
    CrazyflieFw,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic,
    Atan2 => Atan2Logic,
    SinCos => SinCosLogic,
    Sqrt => SqrtLogic,
    QuatMul => QuatMulLogic,
    UnitQuatMul => UnitQuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
    EkfStep => EkfStepLogic,
    LeeController => LeeControllerLogic,
    CrossProduct => CrossProductLogic,
    Vec3Normalize => Vec3NormalizeLogic,
    MatVecMul3x3 => MatVecMul3x3Logic,
    QuatToRotMatrix => QuatToRotMatrixLogic,
    Exp => ExpLogic,
    Ln => LnLogic,
);
