# TASK-652: Integration Tests — Fn Bodies with Let-Sequencing Work End-to-End

## Status: Planned

## Objective

Write integration tests proving that fn bodies with multi-statement let-sequencing work through all code paths: inline fn expressions, top-level fn definitions, and imported pub fn.

## Requirements

1. **Inline fn expression** test:
   ```ash
   workflow main {
       let add = fn(a: Int, b: Int) -> Int {
           let sum = a + b;
           sum
       };
       add(1, 2)
   }
   ```
   Expected result: `3`

2. **Top-level fn definition** test (via engine's `parse_program_with_functions`):
   ```
   fn greet(name: String) -> String {
       let msg = string::concat("Hello, ", name);
       msg
   }
   workflow main { greet("world") }
   ```
   Expected result: `"Hello, world"`

3. **Imported pub fn** test (via `module_loader`):
   - Create a fixture `.ash` file with a `pub fn` that has let-sequencing
   - Import it from another file and call it
   - Verify the result

4. **Nested let-bindings** test:
   ```
   fn fibonacci(n: Int) -> Int {
       let a = 0;
       let b = 1;
       ...
   }
   ```

5. **Pattern matching in let** test:
   ```
   let Some { value: x } = Some { value: 42 };
   x + 1
   ```

6. **Full workspace verification**:
   - `cargo test --workspace` passes
   - `cargo clippy --all-targets --all-features -- -D warnings` clean
   - `cargo fmt --check` clean

## Estimated Hours

1-2

## Completion Checklist

- [ ] Inline fn expression with let-sequencing works
- [ ] Top-level fn definition with let-sequencing works
- [ ] Imported pub fn with let-sequencing works
- [ ] Nested let-bindings work
- [ ] Pattern matching in let works
- [ ] Full workspace gate passes
- [ ] CHANGELOG.md updated
