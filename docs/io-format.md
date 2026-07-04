# Benchmark I/O Format

How benchmark inputs are defined and how results come back out.

Inputs live in one `inputs.json` and are baked into each firmware **at build time** by a
proc-macro, so the `no_std` targets never parse JSON at runtime.

## Input

`benchmarks/inputs.json` lists platform-agnostic cases (the same input runs on every chip):

```json
{
  "repetitions": 1000,
  "cases": [
    {
      "libraries": ["glam"],
      "test": "MatMul3x3",
      "inputs": [
        {
          "lhs": [1, 2, 3, 4, 5, 6, 7, 8, 9],
          "rhs": [9, 8, 7, 6, 5, 4, 3, 2, 1]
        }
      ]
    },

    {
      "libraries": ["nalgebra"],
      "test": "RotateVector",
      "repetitions": 5000,
      "platforms": ["rp2040", "rp2350-arm"],
      "inputs": [{ "point": [1, 2, 3], "quat": [0, 0, 0.7071, 0.7071] }]
    }
  ]
}
```

- `libraries` specifies an array of library implementations to test.
- `test` picks the task identifier, which also fixes the input shape, e.g. `RotateVector` needs `point` (`[f32; 3]`) and `quat` (`[f32; 4]`).
- `repetitions` sets how many times each input is timed. A case may override it.
- `platforms` (optional) restricts a case to some chips. Omit it to run everywhere.

A central registry in `mrs-benchmark-core` maps each task identifier to its implementations and
input type. The `mrs_benchmark_macros::generate_benchmarks!()` proc-macro reads this registry along with `inputs.json` at build time to weave the tasks dynamically into the execution runner for the current platform.

## Output

Each binary logs freely. Result CSV rows stream inline, each prefixed with `BENCH ` so a tool can
grep them out.

```
[some normal log line]
BENCH platform,library,task,input_index,repetitions,duration,unit,result
BENCH stm32,glam,MatMul3x3,0,1000,3003284,cycles,"[30.0, 84.0, 138.0, 24.0, 69.0, 114.0, 18.0, 54.0, 90.0]"
[more logs]
BENCH platform,library,task,input_index,repetitions,duration,unit,result
BENCH host,glam,MatMul3x3,0,1000,104523,ns,"[30.0, 84.0, 138.0, 24.0, 69.0, 114.0, 18.0, 54.0, 90.0]"
```

- Each platform automatically prints the common header row.
- One data row per (platform, library, task, input). Columns, in order:
  `platform,library,task,input_index,repetitions,duration,unit,result`.
- `result` is the actual computed output (e.g. array of floats) formatted as `Debug` so it is easily parsed by Python scripts using `ast.literal_eval`. If the math operation fails (e.g. invalid inverse), the `result` will be `ERROR: <reason>`. The result is enclosed in `""` to prevent its internal commas from breaking the CSV.
- The `duration` represents the total elapsed time for all `repetitions`.
- `platform` and `unit` come directly from the firmware runner (e.g., host outputs `ns`, MCUs output `cycles`).

_Later:_ a host script (not CI-only) greps the `BENCH ` lines, strips the prefix, and merges each platform into one dataset. C libraries slot into the `library` field.
