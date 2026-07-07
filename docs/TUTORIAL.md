# Ash Tutorial

This tutorial is the current productive entry path for Ash. It favors checked examples, standard
profiles, and app scaffolds that are covered by repository gates.

For a deeper app-oriented path, continue with
[Phase 199 Productive Apps Tutorial](tutorials/phase199-productive-apps.md).

## Install And Check

Build the CLI from the repository root:

```bash
cargo build --release
```

Check an Ash file:

```bash
ash check examples/10-testing-helpers/testing_helpers.ash
```

## First Checked Workflow

Ash files use explicit imports and a named entry workflow:

```ash
use test::{assert_true, assert_named}

workflow main {
    let assertion = assert_named("nonzero", true)
    ret assert_true(assertion.passed)
}
```

This shape is intentionally small:

- `use` imports helper values from a standard profile namespace.
- `workflow main` names the entry point.
- `let` binds ordinary values.
- `ret` returns the final value.

The checked version of this pattern lives at
[examples/10-testing-helpers/testing_helpers.ash](../examples/10-testing-helpers/testing_helpers.ash).

## Productive Helper Libraries

Testing helpers live under `std::test` and are exercised by the `phase199_testing_helpers` gate.
The current example imports assertion, property evidence, law evidence, counterexample, coverage,
mutation, flake, provider evidence, deterministic profile, and common fixture helpers.

Process and channel helpers live under `std::process` and are exercised by the
`phase199_process_channel_helpers` gate:

```ash
use process::{spawn_join_plan, bounded_worker_pool, channel_loop_plan}

workflow main {
    let spawn_plan = spawn_join_plan("parallel fetch", 2)
    let pool_plan = bounded_worker_pool("workers", 4, 16)
    let stream_plan = channel_loop_plan("events", "updates", 32)

    ret spawn_plan.preserves_sendability
}
```

The checked process/channel example lives at
[examples/11-process-channel-helpers/process_channel_helpers.ash](../examples/11-process-channel-helpers/process_channel_helpers.ash).

## App Templates

Templates are metadata-driven scaffolds. Start with the canonical manifest index:
[templates/apps/README.md](../templates/apps/README.md).

Instantiate a template with explicit parameters:

```bash
ash template instantiate \
  --manifest templates/apps/cli-tool/template.json \
  --out /tmp/ash-cli-tool \
  --param app_name=demo
```

Template manifests declare required profiles, provider expectations, resources, generated files,
and generated checks. Instantiation validates metadata, writes only relative paths, protects
existing files by default, and runs generated `ash check` commands.

## Authority Model

Ash runtime authority comes from explicit profiles and provider bindings. Templates, examples, and
docs do not grant runtime privileges by themselves. Generated apps must still pass their declared
checks before they are treated as productive artifacts.

## Reference And Migration Material

Long-form design documents and historical examples remain useful for understanding why the language
has its current shape. Treat `docs/spec/`, `docs/reference/`, and older example directories as
reference or migration material unless a tutorial, template, or test explicitly identifies them as a
current productive path.

## Next Steps

1. Read [Phase 199 Productive Apps Tutorial](tutorials/phase199-productive-apps.md).
2. Inspect [templates/apps/README.md](../templates/apps/README.md).
3. Run the checked examples:

```bash
ash check examples/10-testing-helpers/testing_helpers.ash
ash check examples/11-process-channel-helpers/process_channel_helpers.ash
```
