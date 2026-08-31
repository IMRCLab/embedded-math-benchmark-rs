# Adding a new benchmark

A benchmark is a **Task** (an identifier plus a fixed input and output shape) implemented by one
or more **libraries**. The task identifier string has to match in all five places below. You never
touch a platform's `main.rs`: the proc-macro weaves the task into every platform's runner.

## Checklist

1. **Input struct** in `benchmarks/mrs-benchmark-core/src/inputs.rs`, annotated with the task
   identifier. One struct is shared across every library, which is what keeps crates with
   different internal layouts comparable.

   ```rust
   #[derive(Clone, Debug)]
   #[benchmark_input("MyNewTask")]
   pub struct MyNewTaskInput {
       pub value_a: f32,
       pub array_b: [f32; 4],
   }
   ```

   `#[benchmark_input(...)]` reads `benchmarks/inputs.json` at compile time and panics if no case
   exists for that identifier yet, so the crate will not build until step 4 has run at least once.

2. **Task registration** in `mrs-benchmark-core/src/tasks.rs`: an empty struct plus a
   `BenchmarkTask` impl fixing `IDENTIFIER`, `Input` and `Output`. `Output` is the neutral format
   every library must produce, e.g. `[f32; 4]`.

3. **Library implementation** in `mrs-benchmark-core/src/suites/<lib>.rs`, then add the task to
   that file's `export_tasks!` macro. Implement only the libraries you actually wrote; code
   generation skips the rest.

   ```rust
   impl RawTaskImplementation<crate::tasks::MyNewTask> for MyNewTaskLogic {
       type PreparedInput = (f32, glam::Vec4);
       type RawOutput = glam::Vec4;

       // Not timed: convert into the library's own types.
       fn prepare(&self, input: &MyNewTaskInput) -> Self::PreparedInput {
           (input.value_a, glam::Vec4::from_array(input.array_b))
       }

       // Timed. This is the measurement.
       fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
           Ok(input.1 * input.0)
       }

       // Not timed: convert back to the neutral output type.
       fn finalize(&self, output: Self::RawOutput) -> Result<[f32; 4], BenchmarkError> {
           Ok(output.to_array())
       }
   }
   ```

   **`execute` must call the library's own primitive**, including its `try_*` or checked variant
   where one exists, not hand-rolled equivalent math. Besides missing the point of the comparison,
   a hand-rolled guard that duplicates work the library already does internally (computing a
   magnitude by hand and then calling the library's `normalize`, which recomputes it) inflates
   every timed measurement.

4. **Input generator** in `tools/inputs_generator/tasks.py`: a `BenchmarkTask` subclass with
   `generate_inputs` and `compute_reference`, registered in `TASK_REGISTRY`. Append it at the
   **end** of both `TEST_LIBRARY_MAPPING` and `TASK_REGISTRY`; generation shares one seeded RNG
   across all tasks in registry order, so inserting earlier reshuffles every existing task's
   inputs.

   Then `cargo make update`, which regenerates `benchmarks/inputs.json` and the f64
   `reference_results.json` together. Do not hand-edit `inputs.json`; it is generated. Its schema
   is in [io-format.md](io-format.md).

5. **Build and run.** `cargo make bench-host` picks the task up automatically, as does every
   firmware target.
