#pragma once

#include <math.h>
#include "modules/interface/math3d.h"
#include "modules/interface/stabilizer_types.h"

// Declarations of our wrapper functions for static inline math3d.h functions
struct mat33 cf_mmul(struct mat33 a, struct mat33 b);
struct vec cf_qrot(struct quat q, struct vec v);
struct quat cf_qqmul(struct quat q, struct quat p);
struct quat cf_qslerp(struct quat a, struct quat b, float t);

// Lee position/attitude controller (modules/src/controller/controller_lee.c),
// wrapped to take flat inputs and stop at thrust+torque. The firmware's real
// motor mixing is a different, non-sqrt algorithm, so it's out of scope here.
struct cf_lee_output {
    float thrust;
    struct vec torque;
};

struct cf_lee_output cf_lee_controller(
    struct vec position,
    struct vec velocity,
    struct quat attitude,
    struct vec angular_velocity,
    struct vec setpoint_position,
    struct vec setpoint_velocity,
    struct vec setpoint_acceleration,
    float setpoint_yaw,
    float setpoint_yaw_dot,
    float mass
);
