use mrs_benchmark_macros::benchmark_input;

#[benchmark_input("MatMul3x3")]
#[derive(Clone, Debug)]
pub struct MatMul3x3Input {
    pub lhs: [f32; 9],
    pub rhs: [f32; 9],
}

#[benchmark_input("RotateVector")]
#[derive(Clone, Debug)]
pub struct RotateVectorInput {
    pub point: [f32; 3],
    pub quat: [f32; 4], // [x, y, z, w]
}

#[benchmark_input("MatInverse3x3")]
#[derive(Clone, Debug)]
pub struct MatInverse3x3Input {
    pub matrix: [f32; 9],
}

#[benchmark_input("Atan2")]
#[derive(Clone, Debug)]
pub struct Atan2Input {
    pub y: f32,
    pub x: f32,
}

#[benchmark_input("SinCos")]
#[derive(Clone, Debug)]
pub struct SinCosInput {
    pub theta: f32,
}

#[benchmark_input("Sqrt")]
#[derive(Clone, Debug)]
pub struct SqrtInput {
    pub value: f32,
}

#[benchmark_input("QuatMul")]
#[derive(Clone, Debug)]
pub struct QuatMulInput {
    pub lhs: [f32; 4],
    pub rhs: [f32; 4],
}

#[benchmark_input("UnitQuatMul")]
#[derive(Clone, Debug)]
pub struct UnitQuatMulInput {
    pub lhs: [f32; 4],
    pub rhs: [f32; 4],
}

#[benchmark_input("QuatSlerp")]
#[derive(Clone, Debug)]
pub struct QuatSlerpInput {
    pub from: [f32; 4],
    pub to: [f32; 4],
    pub t: f32,
}

#[benchmark_input("LeeController")]
#[derive(Clone, Debug)]
pub struct LeeControllerInput {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub attitude: [f32; 4], // [x, y, z, w]
    pub angular_velocity: [f32; 3],

    pub setpoint_position: [f32; 3],
    pub setpoint_velocity: [f32; 3],
    pub setpoint_acceleration: [f32; 3],
    pub setpoint_yaw: f32,
    pub setpoint_yaw_dot: f32,

    pub mass: f32,
}

#[benchmark_input("EkfStep")]
#[derive(Clone, Debug)]
pub struct EkfStepInput {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub attitude: [f32; 4], // [x, y, z, w]
    pub accelerometer: [f32; 3],
    pub gyroscope: [f32; 3],
    pub range_z: f32,
    pub flow_delta: [f32; 2],
    pub dt: f32,
    pub covariance: [f32; 81],
}

#[benchmark_input("MatMul9x9")]
#[derive(Clone, Debug)]
pub struct MatMul9x9Input {
    pub lhs: [f32; 81],
    pub rhs: [f32; 81],
}

#[benchmark_input("MatInverse9x9")]
#[derive(Clone, Debug)]
pub struct MatInverse9x9Input {
    pub matrix: [f32; 81],
}

#[benchmark_input("DotProduct64D")]
#[derive(Clone, Debug)]
pub struct DotProduct64DInput {
    pub lhs: [f32; 64],
    pub rhs: [f32; 64],
}

#[benchmark_input("CrossProduct")]
#[derive(Clone, Debug)]
pub struct CrossProductInput {
    pub lhs: [f32; 3],
    pub rhs: [f32; 3],
}

#[benchmark_input("Vec3Normalize")]
#[derive(Clone, Debug)]
pub struct Vec3NormalizeInput {
    pub vector: [f32; 3],
}

#[benchmark_input("MatVecMul3x3")]
#[derive(Clone, Debug)]
pub struct MatVecMul3x3Input {
    pub matrix: [f32; 9],
    pub vector: [f32; 3],
}

#[benchmark_input("QuatToRotMatrix")]
#[derive(Clone, Debug)]
pub struct QuatToRotMatrixInput {
    pub quat: [f32; 4], // [x, y, z, w]
}

#[benchmark_input("Exp")]
#[derive(Clone, Debug)]
pub struct ExpInput {
    pub value: f32,
}

#[benchmark_input("Ln")]
#[derive(Clone, Debug)]
pub struct LnInput {
    pub value: f32,
}
