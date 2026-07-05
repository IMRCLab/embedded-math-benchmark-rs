# Benchmark Configuration (`inputs.json`)

The microbenchmark framework uses a JSON-driven, task-centric model. All benchmark inputs, target execution libraries, and target platforms are configured centrally via the `benchmarks/inputs.json` file.

This file is read at compile-time by proc-macros to generate strongly-typed input structures and the global benchmark execution runner (`run_all_benchmarks`).

## Schema Overview

The root of `inputs.json` contains a `cases` array. Each object in this array represents a distinct benchmark task (e.g., `MatMul3x3` or `RotateVector`).

### Case Properties

| Property | Type | Required | Description |
|---|---|---|---|
| `test` | String | Yes | The unique identifier of the benchmark task. This must exactly match the string used in the `#[benchmark_input("TestName")]` macro on your input struct, as well as the `BenchmarkTask` implementation. |
| `repetitions` | Integer | Yes | The number of times the target executor should loop over the `inputs` array when measuring performance. A higher number accumulates more total time, smoothing out noise for extremely fast operations. |
| `libraries` | Array of Strings | Yes | A list of library identifiers (e.g., `"glam"`, `"nalgebra"`, `"micromath"`) to benchmark for this task. The generated runner will dynamically execute the benchmark for exactly the libraries specified here. If a library doesn't implement a benchmark, simply omit it from this list. |
| `platforms` | Array of Strings | No | An optional list of platform IDs (e.g., `"host"`, `"stm32"`, `"rp2040"`). If specified, the generated code will only execute this case if `platform.id()` is in this array. If omitted, the benchmark runs unconditionally on all platforms. |
| `inputs` | Array of Objects | Yes | The actual input data sets for the benchmark. Each object must precisely map its keys to the fields of the Rust `Input` struct defined for the task. The proc-macro uses these values to generate static arrays at compile time. |

## Example `inputs.json`

```json
{
  "cases": [
    {
      "repetitions": 1000,
      "libraries": ["glam", "nalgebra"],
      "test": "MatMul3x3",
      "inputs": [
        {
          "lhs": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
          "rhs": [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0]
        }
      ]
    },
    {
      "repetitions": 10000,
      "libraries": ["glam", "nalgebra", "micromath"],
      "test": "RotateVector",
      "platforms": ["host", "stm32"],
      "inputs": [
        {
          "point": [1.0, 2.0, 3.0],
          "quat": [0.0, 0.0, 0.70710677, 0.70710677]
        }
      ]
    }
  ]
}
```

## How It Works

1. **Input Deserialization**: The `#[benchmark_input("TestName")]` macro searches `inputs.json` for the case where `"test" == "TestName"`. It iterates through the `"inputs"` array, parsing fields to construct `static` arrays inside the Rust code.
2. **Runner Generation**: The `generate_benchmarks!()` macro loops over all cases in `inputs.json`. For every library in `"libraries"`, it emits a `run_and_log` invocation. If `"platforms"` is present, it wraps the invocation in an `if platform.id() == "..."` block.
