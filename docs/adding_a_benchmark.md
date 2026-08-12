# Adding a New Benchmark

This guide outlines the step-by-step process for contributors to add a completely new microbenchmark to the `mrs-microbenchmarks` suite. Our benchmarking framework relies on a highly modular, JSON-driven architecture designed to minimize boilerplate.

## 1. Define the Input Data Structure

First, you need to define what the inputs to your benchmark look like. **Note that this Input struct is shared universally across all libraries** (e.g., `glam`, `nalgebra`). We do this so we can share a single JSON configuration across all libraries and platforms. The specific math operations and conversions to each library's internal memory layout are implemented separately later on.

All input structures live in `benchmarks/mrs-benchmark-core/src/inputs.rs`.

**Build-order gotcha:** `#[benchmark_input(...)]` reads `benchmarks/inputs.json` at compile
time and panics if no case exists yet for that identifier. Do step 4 (or at least a
placeholder case) before your crate will build.

Create a standard Rust `struct`. Annotate your struct with the `#[benchmark_input("MyTestName")]` macro. The name in quotes is the unique **Task Identifier** you will use later.

```rust
// benchmarks/mrs-benchmark-core/src/inputs.rs

use mrs_benchmark_macros::benchmark_input;

#[derive(Clone, Debug)]
#[benchmark_input("MyNewTask")]
pub struct MyNewTaskInput {
    pub value_a: f32,
    pub array_b: [f32; 4],
}
```

## 2. Define the Benchmark Task

Next, register your benchmark as a standard task in the system. This allows the framework to enforce strong typing across different library implementations.

Open `benchmarks/mrs-benchmark-core/src/tasks.rs` and add an empty struct. Implement the `BenchmarkTask` trait for it:

```rust
// benchmarks/mrs-benchmark-core/src/tasks.rs

pub struct MyNewTask;

impl crate::BenchmarkTask for MyNewTask {
    const IDENTIFIER: &'static str = "MyNewTask"; // Must exactly match the macro string!
    type Input = crate::inputs::MyNewTaskInput; // Link to your input struct
    type Output = [f32; 4]; // The standard format your benchmark will produce
}
```

## 3. Implement the Task for a Library

Now you need to actually write the math operations. Go to the suite file of the library you want to benchmark (e.g., `benchmarks/mrs-benchmark-core/src/suites/glam.rs`).

**Call the library's own function, not your own math.** The point of this project is
comparing libraries, so `execute` (and any domain-error guard it needs, e.g. "don't
normalize a near-zero vector") should call that library's real primitive — including its
`try_*`/checked variant if one exists — rather than hand-rolled equivalent math. Besides
missing the point of the comparison, a hand-rolled check that duplicates work the library's
own call already does internally (e.g. computing a magnitude by hand *and* then calling the
library's normalize, which recomputes it) silently inflates every timed measurement.

Create a "Logic" struct and implement `RawTaskImplementation`:

```rust
// inside e.g., glam.rs

pub struct MyNewTaskLogic;

impl RawTaskImplementation<crate::tasks::MyNewTask> for MyNewTaskLogic {
    // What the internal glam objects look like
    type PreparedInput = (f32, glam::Vec4); 
    type RawOutput = glam::Vec4;

    fn prepare(&self, input: &<crate::tasks::MyNewTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        // Convert from generic raw inputs to glam-specific types (Not timed!)
        (input.value_a, glam::Vec4::from_array(input.array_b))
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, crate::BenchmarkError> {
        // The actual math operation you want to measure (Timed!)
        Ok(input.1 * input.0)
    }

    fn finalize(&self, output: Self::RawOutput) -> Result<<crate::tasks::MyNewTask as crate::BenchmarkTask>::Output, crate::BenchmarkError> {
        // Convert from glam types back to standard Rust types (Not timed!)
        Ok(output.to_array())
    }
}
```

Finally, at the bottom of the suite file (e.g., `glam.rs`), add your new task to the `export_tasks!` macro:

```rust
export_tasks!(
    Glam,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic,
    MyNewTask => MyNewTaskLogic // <--- ADD YOUR TASK HERE
);
```

## 4. Add the Configuration to `inputs.json`

The final step is to tell the runner to execute your benchmark by adding it to `benchmarks/inputs.json`.

**In this repo, don't hand-edit `inputs.json` directly** — it's generated from
`tools/inputs_generator/tasks.py`'s `TASK_REGISTRY` (a `BenchmarkTask` subclass with
`generate_inputs`/`compute_reference`, appended at the *end* of `TEST_LIBRARY_MAPPING` and
`TASK_REGISTRY` — inserting earlier reshuffles every existing task's random inputs, since
generation shares one seeded RNG across all tasks in registry order). Run `cargo make
inputs` (then `cargo make generate-ref`) to regenerate it. The shape below is illustrative
of what ends up in the file, not something you write by hand.

Add a new object to the `cases` array. Ensure `"test"` matches your identifier, and the keys inside `"inputs"` exactly match the fields of your `MyNewTaskInput` struct.

```json
{
  "cases": [
    {
      "repetitions": 1000,
      "libraries": ["glam"],
      "test": "MyNewTask",
      "inputs": [
        {
          "value_a": 5.0,
          "array_b": [1.0, 2.0, 3.0, 4.0]
        },
        {
          "value_a": 2.5,
          "array_b": [0.0, 0.0, 1.0, 0.0]
        }
      ]
    }
    // ... other cases
  ]
}
```

### Note on Libraries

You don't have to implement your new task for every single math library! Just list the libraries you *did* write implementations for in the `"libraries"` array in the JSON file. The code generation will gracefully skip the ones you omitted.

## 5. Build and Run!

You are done! You do not need to touch the `main.rs` files for the host, STM32, or RP2040. The proc-macro will automatically detect your JSON additions, generate the static data arrays, and weave your benchmark into the global runner loops for every platform.
