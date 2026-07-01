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
      "library": "glam",
      "test": "MatMul3x3",
      "inputs": [
        {
          "lhs": [1, 2, 3, 4, 5, 6, 7, 8, 9],
          "rhs": [9, 8, 7, 6, 5, 4, 3, 2, 1]
        }
      ]
    },

    {
      "library": "nalgebra",
      "test": "RotateVector",
      "repetitions": 5000,
      "platforms": ["rp2040", "rp2350-arm"],
      "inputs": [{ "point": [1, 2, 3], "quat": [0, 0, 0.7071, 0.7071] }]
    }
  ]
}
```

- `library` + `test` pick the implementation. `test` also fixes the input shape, e.g.
  `RotateVector` needs `point` (`[f32; 3]`) and `quat` (`[f32; 4]`).
- `repetitions` sets how many times each input is timed. A case may override it.
- `platforms` (optional) restricts a case to some chips. Omit it to run everywhere.

A central registry in `mrs-benchmark-core` maps each `(library, test)` to its impl and
input type. The proc-macro reads that registry together with the JSON, so the names in the
file and the types in the code stay in sync.

_proc-macro sketch proposol, to be decided:_ At build time it parses `inputs.json`,
looks each case up in the registry (strings like `"glam"` / `"MatMul3x3"` map to the idents
`GlamMatMul3x3` / `MatrixMul3x3Input`), and emits plain Rust: the input struct built as
literals, a rep loop that keeps the min, and the `BENCH ` line. The call site passes the JSON
path, the platform, and the registry it needs to turn names into idents:

```rust
run_benchmarks! {
    inputs: "benchmarks/inputs.json",
    platform: HostPlatform,
    registry: {
        MatMul3x3    => MatrixMul3x3Input { glam: GlamMatMul3x3, nalgebra: NAlgMatMul3x3 },
        RotateVector => RotateVectorInput { glam: GlamRotateVector, nalgebra: NAlgRotateVector },
    },
}
```

## Output

Each binary logs freely. Result CSV rows stream inline, each prefixed with `BENCH ` so a tool can
grep them out.

```
[some normal log line]
BENCH MatMul3x3,glam,rp2040,0,1000,142,cycles
[more logs]
BENCH MatMul3x3,glam,host,0,1000,41,ns
```

- One row per (test, library, platform, input). Columns, in order:
  `test,library,platform,input_index,reps,min_duration,unit`.
- No header line. The firmware never prints one; the column order above is the contract, and
  the host tool prepends the header when it builds the CSV.
- `platform` and `unit` come from the firmware, not the input.
- `min_duration` is the minimum over `reps`, the stablest number for deterministic code.
- `unit` is native for now: `ns` on the host, `cycles` on the MCUs.

_Later:_ a host script (not CI-only) greps the `BENCH ` lines, strips the prefix, adds the
header, and merges each platform into one dataset. C libraries slot into the `library` field.
