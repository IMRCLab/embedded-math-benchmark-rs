"""Report-wide constants: library/platform display order and styling, task display
order, grid shapes, and the per-platform clock rates used to convert firmware cycle
counts to nanoseconds."""

# Fixed order + validated colorblind-safe palette (dataviz skill, categorical
# slots 1-5). crazyflie-fw is the only C impl: distinct hue, always rightmost,
# plus a hatch so it reads as "different" even in grayscale/print.
LIBRARY_ORDER = ["glam", "libm", "micromath", "nalgebra", "crazyflie-fw"]
LIBRARY_COLORS = {
    "glam": "#2a78d6",
    "libm": "#eb6834",
    "micromath": "#1baf7a",
    "nalgebra": "#eda100",
    "crazyflie-fw": "#e87ba4",
}
CF_HATCH = "///"
FALLBACK_COLOR = "#999999"

# Display order mirrors benchmarks/inputs.json case order. Kept by hand (like
# LIBRARY_ORDER above) rather than read from inputs.json: a results.csv may be
# reviewed without the Rust tree next to it, and collect's merged CSV rows are
# already alphabetically sorted so there's no order left to recover from the
# data itself. A task present in the data but missing here is appended
# (alphabetically) rather than silently dropped.
TASK_ORDER = [
    "MatMul3x3",
    "RotateVector",
    "MatInverse3x3",
    "Atan2",
    "SinCos",
    "Sqrt",
    "QuatMul",
    "UnitQuatMul",
    "QuatSlerp",
    "LeeController",
]

# MCUs in CLAUDE.md's crate map order, host last: it's not an interesting reference
# point next to real hardware, so it trails instead of leading every page/table.
PLATFORM_ORDER = ["stm32", "rp2040", "rp2350-arm", "esp32s3", "host"]

LOG_THRESHOLD = 10.0
LOG_COLOR = "#c62828"
GRID_SHAPE = (2, 4)  # rows, cols per page; extra tasks spill onto page 2, 3, ...
TASK_GRID_SHAPE = (2, 2)  # by-task pages: fewer, wider subplots for the platform x library groups
UNIT = "ns"

# Empty space (in slot-widths) between platform groups within a by-task subplot.
GROUP_GAP = 2.5

# Cycle counters read raw core cycles; converting to time needs each
# platform's clock rate (see the `main.rs` of the corresponding
# mrs-benchmark-<platform> crate for where each is configured). host already
# reports `ns` directly via std::time::Instant and needs no conversion.
PLATFORM_CLOCK_HZ = {
    "stm32": 168e6,
    "rp2040": 125e6,
    "rp2350-arm": 150e6,
    "esp32s3": 240e6,
}
