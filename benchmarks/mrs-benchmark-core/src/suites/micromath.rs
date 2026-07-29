use crate::tasks::{
    Atan2 as Atan2Task, LeeController as LeeControllerTask, QuatMul as QuatMulTask,
    QuatSlerp as QuatSlerpTask, RotateVector as RotateVectorTask, SinCos as SinCosTask,
    Sqrt as SqrtTask, UnitQuatMul as UnitQuatMulTask,
};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};
use micromath::{F32Ext, Quaternion};

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

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let q = &input.0;
        let p = &input.1;
        let q_vec = Quaternion::new(0.0, p[0], p[1], p[2]);
        Ok(*q * q_vec * q.conj())
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<RotateVectorTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x(), output.y(), output.z()])
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
        Ok(input.0.atan2(input.1))
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
        Ok((input.sin(), input.cos()))
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
            Ok(input.sqrt())
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
    type PreparedInput = (Quaternion, Quaternion);
    type RawOutput = Quaternion;

    fn prepare(&self, input: &<QuatMulTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let lhs = Quaternion::new(input.lhs[3], input.lhs[0], input.lhs[1], input.lhs[2]);
        let rhs = Quaternion::new(input.rhs[3], input.rhs[0], input.rhs[1], input.rhs[2]);
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x(), output.y(), output.z(), output.w()])
    }
}

pub struct UnitQuatMulLogic;
impl RawTaskImplementation<UnitQuatMulTask> for UnitQuatMulLogic {
    type PreparedInput = (Quaternion, Quaternion);
    type RawOutput = Quaternion;

    fn prepare(
        &self,
        input: &<UnitQuatMulTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let lhs =
            Quaternion::new(input.lhs[3], input.lhs[0], input.lhs[1], input.lhs[2]).normalize();
        let rhs =
            Quaternion::new(input.rhs[3], input.rhs[0], input.rhs[1], input.rhs[2]).normalize();
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<UnitQuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x(), output.y(), output.z(), output.w()])
    }
}

pub struct QuatSlerpLogic;
impl RawTaskImplementation<QuatSlerpTask> for QuatSlerpLogic {
    type PreparedInput = (Quaternion, Quaternion, f32);
    type RawOutput = Quaternion;

    fn prepare(
        &self,
        input: &<QuatSlerpTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let from = Quaternion::new(input.from[3], input.from[0], input.from[1], input.from[2]);
        let to = Quaternion::new(input.to[3], input.to[0], input.to[1], input.to[2]);
        (from, to, input.t)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let q1 = input.0;
        let mut q2 = input.1;
        let t = input.2;

        let mut dot = q1.x() * q2.x() + q1.y() * q2.y() + q1.z() * q2.z() + q1.w() * q2.w();

        if dot < 0.0 {
            q2 = Quaternion::new(-q2.w(), -q2.x(), -q2.y(), -q2.z());
            dot = -dot;
        }

        const DOT_THRESHOLD: f32 = 0.9995;
        if dot > DOT_THRESHOLD {
            let x = q1.x() + t * (q2.x() - q1.x());
            let y = q1.y() + t * (q2.y() - q1.y());
            let z = q1.z() + t * (q2.z() - q1.z());
            let w = q1.w() + t * (q2.w() - q1.w());
            let len = (x * x + y * y + z * z + w * w).sqrt();
            return Ok(Quaternion::new(w / len, x / len, y / len, z / len));
        }

        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();

        let s0 = (theta_0 - theta).sin() / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        let x = s0 * q1.x() + s1 * q2.x();
        let y = s0 * q1.y() + s1 * q2.y();
        let z = s0 * q1.z() + s1 * q2.z();
        let w = s0 * q1.w() + s1 * q2.w();

        Ok(Quaternion::new(w, x, y, z))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatSlerpTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x(), output.y(), output.z(), output.w()])
    }
}

// --- Tiny Math Wrapper for LeeControllerSim in Micromath ---
#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    fn dot(&self, o: &Self) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    fn cross(&self, o: &Self) -> Self {
        Self {
            x: self.y * o.z - self.z * o.y,
            y: self.z * o.x - self.x * o.z,
            z: self.x * o.y - self.y * o.x,
        }
    }
    fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }
    fn normalize(&self) -> Self {
        let n = self.length();
        Self {
            x: self.x / n,
            y: self.y / n,
            z: self.z / n,
        }
    }
}
impl core::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
        }
    }
}
impl core::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self {
            x: self.x - o.x,
            y: self.y - o.y,
            z: self.z - o.z,
        }
    }
}
impl core::ops::Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}
impl core::ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        v * self
    }
}
impl core::ops::Mul<Vec3> for Vec3 {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self {
            x: self.x * o.x,
            y: self.y * o.y,
            z: self.z * o.z,
        }
    }
}
impl core::ops::Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Mat3 {
    pub x_axis: Vec3,
    pub y_axis: Vec3,
    pub z_axis: Vec3,
}
impl Mat3 {
    fn from_cols(x_axis: Vec3, y_axis: Vec3, z_axis: Vec3) -> Self {
        Self {
            x_axis,
            y_axis,
            z_axis,
        }
    }
    fn transpose(&self) -> Self {
        Self {
            x_axis: Vec3::new(self.x_axis.x, self.y_axis.x, self.z_axis.x),
            y_axis: Vec3::new(self.x_axis.y, self.y_axis.y, self.z_axis.y),
            z_axis: Vec3::new(self.x_axis.z, self.y_axis.z, self.z_axis.z),
        }
    }
    fn from_quat(q: Quaternion) -> Self {
        let x2 = q.x() + q.x();
        let y2 = q.y() + q.y();
        let z2 = q.z() + q.z();
        let xx = q.x() * x2;
        let xy = q.x() * y2;
        let xz = q.x() * z2;
        let yy = q.y() * y2;
        let yz = q.y() * z2;
        let zz = q.z() * z2;
        let wx = q.w() * x2;
        let wy = q.w() * y2;
        let wz = q.w() * z2;
        Self {
            x_axis: Vec3::new(1.0 - (yy + zz), xy + wz, xz - wy),
            y_axis: Vec3::new(xy - wz, 1.0 - (xx + zz), yz + wx),
            z_axis: Vec3::new(xz + wy, yz - wx, 1.0 - (xx + yy)),
        }
    }
}
impl core::ops::Mul<Vec3> for Mat3 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        self.x_axis * v.x + self.y_axis * v.y + self.z_axis * v.z
    }
}
impl core::ops::Mul<Mat3> for Mat3 {
    type Output = Mat3;
    fn mul(self, m: Mat3) -> Mat3 {
        Self {
            x_axis: self * m.x_axis,
            y_axis: self * m.y_axis,
            z_axis: self * m.z_axis,
        }
    }
}

impl core::ops::Sub for Mat3 {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self {
            x_axis: self.x_axis - o.x_axis,
            y_axis: self.y_axis - o.y_axis,
            z_axis: self.z_axis - o.z_axis,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl Vec4 {
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}
impl core::ops::Mul<f32> for Vec4 {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
            w: self.w * s,
        }
    }
}
impl core::ops::Add for Vec4 {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
            w: self.w + o.w,
        }
    }
}

pub struct LeeControllerLogic;

impl RawTaskImplementation<LeeControllerTask> for LeeControllerLogic {
    type PreparedInput = (
        Vec3,
        Vec3,
        Quaternion,
        Vec3, // state
        Vec3,
        Vec3,
        Vec3,
        f32,
        f32, // cmd
        f32, // mass
    );
    type RawOutput = [f32; 4];

    fn prepare(
        &self,
        input: &<LeeControllerTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let q = Quaternion::new(
            input.attitude[3],
            input.attitude[0],
            input.attitude[1],
            input.attitude[2],
        );
        (
            Vec3::new(input.position[0], input.position[1], input.position[2]),
            Vec3::new(input.velocity[0], input.velocity[1], input.velocity[2]),
            q,
            Vec3::new(
                input.angular_velocity[0],
                input.angular_velocity[1],
                input.angular_velocity[2],
            ),
            Vec3::new(
                input.setpoint_position[0],
                input.setpoint_position[1],
                input.setpoint_position[2],
            ),
            Vec3::new(
                input.setpoint_velocity[0],
                input.setpoint_velocity[1],
                input.setpoint_velocity[2],
            ),
            Vec3::new(
                input.setpoint_acceleration[0],
                input.setpoint_acceleration[1],
                input.setpoint_acceleration[2],
            ),
            input.setpoint_yaw,
            input.setpoint_yaw_dot,
            input.mass,
        )
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let (pos, vel, att, ang_vel, cmd_pos, cmd_vel, cmd_acc, cmd_yaw, cmd_yaw_rate, mass) =
            *input;

        use crate::suites::constants::*;
        let kp = Vec3::new(KP[0], KP[1], KP[2]);
        let kd = Vec3::new(KD[0], KD[1], KD[2]);
        let k_r = Vec3::new(K_R[0], K_R[1], K_R[2]);
        let k_w = Vec3::new(K_W[0], K_W[1], K_W[2]);
        let gravity = Vec3::new(GRAVITY[0], GRAVITY[1], GRAVITY[2]);
        let inertia = Vec3::new(INERTIA[0], INERTIA[1], INERTIA[2]);

        let e_p = cmd_pos - pos;
        let e_v = cmd_vel - vel;

        let f_d = mass * (cmd_acc + kp * e_p + kd * e_v - gravity);

        let r_mat = Mat3::from_quat(att);
        let thrust = f_d.dot(&(r_mat * Vec3::new(0.0, 0.0, 1.0)));

        let xcd = Vec3::new(cmd_yaw.cos(), cmd_yaw.sin(), 0.0);
        let ycd = Vec3::new(-cmd_yaw.sin(), cmd_yaw.cos(), 0.0);

        let xbd = ycd.cross(&f_d).normalize();
        let ybd = f_d.cross(&xbd).normalize();
        let zbd = xbd.cross(&ybd);
        let r_d = Mat3::from_cols(xbd, ybd, zbd);

        let r_d_t_r = r_d.transpose() * r_mat;
        let r_t_r_d = r_mat.transpose() * r_d;
        let err_mat = r_d_t_r - r_t_r_d;
        // vee map of skew symmetric matrix: (M_32, M_13, M_21)
        let e_r = 0.5 * Vec3::new(err_mat.y_axis.z, err_mat.z_axis.x, err_mat.x_axis.y);

        let w_d = if f_d.length() > f32::EPSILON {
            let cmd_jerk = Vec3::new(0.0, 0.0, 0.0);
            let c = zbd.dot(&(cmd_acc - gravity));
            let d1 = xbd.dot(&cmd_jerk);
            let d2 = -ybd.dot(&cmd_jerk);
            let d3 = cmd_yaw_rate * xcd.dot(&xbd);

            let b3 = -ycd.dot(&zbd);
            let c3 = ycd.cross(&zbd).length();

            let wxd = d2 / c;
            let wyd = d1 / c;
            let wzd = (c * d3 - b3 * d1) / (c * c3);
            Vec3::new(wxd, wyd, wzd)
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        };

        let e_w = ang_vel - r_mat.transpose() * r_d * w_d;

        let feedback = -k_r * e_r - k_w * e_w;
        let gyro = ang_vel.cross(&(inertia * ang_vel));
        let feed_forward = -inertia * (ang_vel.cross(&(r_mat.transpose() * r_d * w_d)));
        let torque = feedback + gyro + feed_forward;

        // --- Allocation ---
        let kappa_f = KAPPA_F;
        let kappa_tau = KAPPA_TAU;
        let a = A;

        let inv_k_f = 1.0 / (4.0 * kappa_f);
        let inv_k_t_xy = 1.0 / (4.0 * kappa_f * a);
        let inv_k_t_z = 1.0 / (4.0 * kappa_tau);

        let w = Vec4::new(thrust, torque.x, torque.y, torque.z);
        let c0 = Vec4::new(inv_k_f, inv_k_f, inv_k_f, inv_k_f);
        let c1 = Vec4::new(-inv_k_t_xy, -inv_k_t_xy, inv_k_t_xy, inv_k_t_xy);
        let c2 = Vec4::new(-inv_k_t_xy, inv_k_t_xy, inv_k_t_xy, -inv_k_t_xy);
        let c3 = Vec4::new(-inv_k_t_z, inv_k_t_z, -inv_k_t_z, inv_k_t_z);

        let m1234_sq = c0 * w.x + c1 * w.y + c2 * w.z + c3 * w.w;

        let motors = Vec4::new(
            m1234_sq.x.max(0.0).sqrt(),
            m1234_sq.y.max(0.0).sqrt(),
            m1234_sq.z.max(0.0).sqrt(),
            m1234_sq.w.max(0.0).sqrt(),
        );

        Ok([motors.x, motors.y, motors.z, motors.w])
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<LeeControllerTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

export_tasks!(
    Micromath,
    RotateVector => RotateVectorLogic,
    Atan2 => Atan2Logic,
    SinCos => SinCosLogic,
    Sqrt => SqrtLogic,
    QuatMul => QuatMulLogic,
    UnitQuatMul => UnitQuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
    LeeController => LeeControllerLogic,
);
