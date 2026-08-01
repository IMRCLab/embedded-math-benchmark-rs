#include <math.h>
#include <string.h>

#include "math3d.h"
#include "controller_lee.h"

// controllerLee() only calls this on the setpoint branch we never take
// (setpoint->mode.{x,y,z} is always modeAbs below); this stub only
// satisfies the linker, it is never executed.
float powerDistributionGetMaxThrust(void) {
    return 0.0f;
}

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
) {
    controllerLee_t self;
    controllerLeeInit(&self);
    self.mass = mass;
    // Benchmark-only override: the other 3 implementations only model
    // P + D + feedforward. A single call from a fresh reset still
    // accumulates one dt-step of attitude integral before using it, so
    // zero the gain to keep the comparison apples-to-apples.
    self.KI = vzero();

    setpoint_t setpoint;
    memset(&setpoint, 0, sizeof(setpoint));
    setpoint.mode.x = modeAbs;
    setpoint.mode.y = modeAbs;
    setpoint.mode.z = modeAbs;
    setpoint.mode.yaw = modeAbs;
    setpoint.attitude.yaw = degrees(setpoint_yaw);
    setpoint.attitudeRate.yaw = degrees(setpoint_yaw_dot);
    setpoint.position.x = setpoint_position.x;
    setpoint.position.y = setpoint_position.y;
    setpoint.position.z = setpoint_position.z;
    setpoint.velocity.x = setpoint_velocity.x;
    setpoint.velocity.y = setpoint_velocity.y;
    setpoint.velocity.z = setpoint_velocity.z;
    setpoint.acceleration.x = setpoint_acceleration.x;
    setpoint.acceleration.y = setpoint_acceleration.y;
    setpoint.acceleration.z = setpoint_acceleration.z;

    sensorData_t sensors;
    memset(&sensors, 0, sizeof(sensors));
    sensors.gyro.x = degrees(angular_velocity.x);
    sensors.gyro.y = degrees(angular_velocity.y);
    sensors.gyro.z = degrees(angular_velocity.z);

    state_t state;
    memset(&state, 0, sizeof(state));
    state.position.x = position.x;
    state.position.y = position.y;
    state.position.z = position.z;
    state.velocity.x = velocity.x;
    state.velocity.y = velocity.y;
    state.velocity.z = velocity.z;
    state.attitudeQuaternion.x = attitude.x;
    state.attitudeQuaternion.y = attitude.y;
    state.attitudeQuaternion.z = attitude.z;
    state.attitudeQuaternion.w = attitude.w;

    control_t control;
    memset(&control, 0, sizeof(control));

    controllerLee(&self, &control, &setpoint, &sensors, &state, 0);

    struct cf_lee_output out;
    out.thrust = control.thrustSi;
    out.torque = mkvec(control.torque[0], control.torque[1], control.torque[2]);
    return out;
}
