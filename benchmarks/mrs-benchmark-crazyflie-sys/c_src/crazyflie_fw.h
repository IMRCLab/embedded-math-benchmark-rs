#pragma once

#include <math.h>
#include "modules/interface/math3d.h"
#include "modules/interface/stabilizer_types.h"

// Declarations of our wrapper functions for static inline math3d.h functions
struct mat33 cf_mmul(struct mat33 a, struct mat33 b);
struct vec cf_qrot(struct quat q, struct vec v);
struct quat cf_qqmul(struct quat q, struct quat p);
struct quat cf_qslerp(struct quat a, struct quat b, float t);
