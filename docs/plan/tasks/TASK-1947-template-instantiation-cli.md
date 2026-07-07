# TASK-1947: Template Instantiation CLI

**Status:** Complete
**Phase:** [PLAN-199: Productive App Libraries And Templates](../PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)

## Description

Add a CLI/template instantiation path with structured fail-closed diagnostics.

## Requirements

- Instantiate validated templates into target project or fixture layouts.
- Support parameter substitution only through typed, validated template parameters.
- Refuse to overwrite files unless an explicit safe overwrite mode is specified.
- Run template conformance checks after instantiation.

## TDD Steps

1. Add failing CLI/template instantiation tests.
2. Implement minimal instantiation and diagnostics.
3. Add overwrite, parameter, and conformance failure tests.
4. Run focused CLI/template tests and Rust quality gates.

## Completion Checklist

- [x] Template CLI path instantiates validated templates.
- [x] Parameters are typed and validated.
- [x] Existing files are protected by default.
- [x] Post-instantiation checks run and report structured diagnostics.

## Evidence

- Added `ash template instantiate --manifest <path> --out <dir> --param key=value`, backed by
  `ash_cli::templates::validate_template_manifest`.
- Instantiation substitutes only declared manifest parameters, refuses missing required parameters,
  rejects undeclared parameter keys, and protects existing files unless `--overwrite` is supplied.
- Generated checks currently support declared `ash check <file>` commands and run after files are
  written, so invalid generated Ash fails the command.
- Focused verification:
  `cargo test -p ash-cli --test phase199_template_instantiation_cli -- --nocapture`.
