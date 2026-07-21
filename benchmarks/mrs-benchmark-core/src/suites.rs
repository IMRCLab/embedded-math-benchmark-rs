pub mod crazyflie_fw;
pub mod glam;
pub mod libm;
pub mod micromath;
pub mod nalgebra;

pub mod constants {
    pub const KP: [f32; 3] = [7.0, 7.0, 7.0];
    pub const KD: [f32; 3] = [4.0, 4.0, 4.0];
    pub const K_R: [f32; 3] = [0.007, 0.007, 0.008];
    pub const K_W: [f32; 3] = [0.00115, 0.00115, 0.002];
    pub const GRAVITY: [f32; 3] = [0.0, 0.0, -9.80665];
    pub const INERTIA: [f32; 3] = [16.571710e-6, 16.655602e-6, 29.261652e-6];

    pub const KAPPA_F: f32 = 2.2168774885271255e-10;
    pub const KAPPA_TAU: f32 = 1.2096801313292268e-12;
    pub const A: f32 = 0.046 / core::f32::consts::SQRT_2;
}
