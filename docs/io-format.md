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

- `library` + `test` pick the implementation. `test` also fixes the input shape:
  `MatMul3x3` needs `lhs`/`rhs` (`[f32; 9]`, row-major); `RotateVector` needs `point`
  (`[f32; 3]`) and `quat` (`[f32; 4]`, ordered `[x, y, z, w]`).
- `repetitions` sets how many times each input is timed. A case may override it.
- `platforms` (optional) restricts a case to some chips. Omit it to run everywhere.

A central registry in `mrs-benchmark-core` maps each `(library, test)` to its impl and
input type. The proc-macro reads that registry together with the JSON, so the names in the
file and the types in the code stay in sync.

## Output

Each binary logs freely, then prints one CSV block between markers so a tool can pull it out.

```
=== CSV Begin ===
test,library,platform,input_index,reps,min_duration,unit
MatMul3x3,glam,rp2040,0,1000,142,cycles
MatMul3x3,glam,host,0,1000,41,ns
=== CSV End ===
```

- One row per (test, library, platform, input). The header sits inside the markers, so the
  block is a complete CSV.
- `platform` and `unit` come from the firmware, not the input.
- `min_duration` is the minimum over `reps`, the stablest number for deterministic code.
- `unit` is native for now: `ns` on the host, `cycles` on the MCUs.

_Later:_ a host tool merges each platform's CSV block into one dataset; C libraries slot into
the `library` field.
