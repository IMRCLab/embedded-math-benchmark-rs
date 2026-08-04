#pragma once
// crazyflie-firmware's platform_defaults.h includes this unconditionally.
// It's normally Kconfig-generated at firmware build time; we don't run
// that build system here. Leaving it empty makes platform_defaults.h fall
// through to its own #ifndef CF_MASS / ARM_LENGTH / ... defaults, which is
// fine: controller_lee.c only needs CF_MASS for a static initializer this
// wrapper immediately overrides with the benchmark's own `mass` input.
