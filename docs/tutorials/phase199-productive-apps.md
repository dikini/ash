# Phase 199 Productive Apps Tutorial

This tutorial shows the productive Phase 199 path for app-oriented Ash work. The goal is to start
from validated standard profiles, use current helper libraries, and generate app skeletons only
after template metadata passes validation.

## Read Path

1. Review the canonical template index:
   [templates/apps/README.md](../../templates/apps/README.md).
2. Inspect the testing helper example:
   [examples/10-testing-helpers/testing_helpers.ash](../../examples/10-testing-helpers/testing_helpers.ash).
3. Inspect the process and channel helper example:
   [examples/11-process-channel-helpers/process_channel_helpers.ash](../../examples/11-process-channel-helpers/process_channel_helpers.ash).
4. Read the manifest schema:
   [phase-199-app-template-manifest-schema.md](../reference/phase-199-app-template-manifest-schema.md).

## Generate A Template

Use the template CLI after choosing a manifest from `templates/apps`:

```bash
ash template instantiate \
  --manifest templates/apps/cli-tool/template.json \
  --out /tmp/ash-cli-tool \
  --param app_name=demo
```

The command validates `ash-template-v1` metadata, substitutes declared parameters, writes relative
files only, protects existing files by default, and runs generated `ash check` commands.

## Helper Examples

Testing helpers live under `std::test`. The gated example imports assertion, property, law,
counterexample, coverage, mutation, flake, provider evidence, deterministic profile, and common
fixture helpers. Its focused gate is `phase199_testing_helpers`.

Process and channel helpers live under `std::process`. The gated example imports spawn/join plan,
bounded worker pool, channel-loop plan, cancellation cleanup, sendability guard, channel diagnostic,
and process trace helper records. Its focused gate is `phase199_process_channel_helpers`.

## Template Gates

The canonical template corpus is checked by `phase199_canonical_templates`. That gate discovers the
five canonical manifests, instantiates each through `ash template instantiate`, supplies the required
`app_name` parameter, and relies on the generated `ash check` commands for conformance.

Use `phase199_template_manifest` for schema validation and `phase199_template_instantiation_cli` for
CLI behavior such as parameter admission, overwrite protection, and generated-check failure
reporting.

## Authority Model

Templates describe required profiles, provider operations, resources, and evidence expectations.
They do not install providers, widen admission, or create new runtime privileges. Runtime authority
still comes from explicit profiles and provider bindings.
