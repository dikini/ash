---
id: ref.tools.cli
title: Ash CLI Command Map
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: cli
last_verified: 2026-06-01
verified_against:
  git_commit: e06944a
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-005-CLI.md
    - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-994-reference-getting-started-journey.md
    - docs/plan/tasks/TASK-995-reference-ashgrove-and-cli-procedures.md
  code:
    - crates/ash-cli/src/main.rs
    - crates/ash-cli/src/commands/run.rs
    - crates/ash-cli/src/commands/daemon.rs
  tests:
    - cargo run -p ash-cli -- --help
    - cargo run -p ash-cli -- check --help
    - cargo run -p ash-cli -- run --help
    - cargo run -p ash-cli -- trace --help
    - cargo run -p ash-cli -- test --help
    - cargo run -p ash-cli -- repl --help
    - cargo run -p ash-cli -- dot --help
    - cargo run -p ash-cli -- daemon --help
    - check_frontmatter full reference validation
  examples:
    []
related:
  depends_on:
    - ref.tools.index
  explains:
    - ref.getting_started.run_a_program
    - ref.getting_started.run_as_daemon
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/design/DESIGN-043-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
refresh_trigger:
  - docs/spec/SPEC-005-CLI.md changes
  - docs/spec/SPEC-070-ALPHA-RUNTIME-KERNEL-AND-OS-SURFACE.md changes
  - crates/ash-cli/src/** changes
  - reference/tools/cli.md changes
---

# Ash CLI Command Map

`ash` is the language and runtime CLI. Ashgrove installation, update, dependency locking, vendoring, and cleanup procedures live in the [Ashgrove reference](ashgrove.md).

The command map below was checked against live help. Command snippets with placeholders such as `PATH` are help-derived reference forms, not copy/paste examples.

## Top-Level Form

```bash
cargo run -p ash-cli -- --help
```

Help-derived installed form:

```bash
ash [OPTIONS] <COMMAND>
```

Global options currently include `--quiet`, `--color <auto|always|never>`, repeatable `-v`, `--help`, and `--version`.

## Commands

| Command | Help summary | Primary use |
| --- | --- | --- |
| `ash check <PATH>` | Type check workflow files | Validate one file or a directory, optionally recursively. |
| `ash run <PATH> [-- <ARGS>...]` | Execute a workflow | One-shot local execution with output, trace, dry-run, timeout, capability/resource, and admission-profile options. |
| `ash trace <PATH>` | Run workflow with provenance tracing | Produce trace data in JSON/NDJSON/CSV or export forms. |
| `ash test [PATH]` | Run Ash tests | Run file/directory tests with filters and synthesized-test controls. |
| `ash repl` | Start interactive REPL | Start an interactive session with optional history, init, and config paths. |
| `ash dot <PATH>` | Generate Graphviz DOT output | Emit DOT or SVG graph output for a workflow file. |
| `ash daemon <COMMAND>` | Control the local RuntimeKernel daemon | Serve and inspect the local same-user daemon surface. |

## Check

Reference-only command forms from `ash check --help`:

```bash
ash check PATH
ash check --all PATH
ash check --strict --format json PATH
ash check --policy-check PATH
```

`PATH` may be a workflow file or directory. `--all` recursively checks files in a directory. `--strict` treats warnings as errors. `--format` accepts `human` or `json`.

## Run

Reference-only command forms from `ash run --help`:

```bash
ash run PATH
ash run --dry-run PATH
ash run --trace --format json PATH
ash run --timeout SECONDS PATH
ash run --admission-profile allow PATH
ash run PATH -- ARG...
```

`--admission-profile` accepts `empty`, `allow`, or `reject`. Capability and resource bindings use the help-documented shapes `BINDING=IMPLEMENTATION` and `RESOURCE=INITIALIZER`.

## Trace

Reference-only command forms from `ash trace --help`:

```bash
ash trace PATH
ash trace --format ndjson PATH
ash trace --lineage --verify PATH
ash trace --export provn PATH
```

The help surface also exposes `--sign`, `--provn`, and `--cypher`. This page does not claim a production signing or provenance storage policy beyond the current CLI surface.

## Test, REPL, and Dot

Reference-only command forms:

```bash
ash test PATH
ash test --tag TAG --kind KIND PATH
ash repl --history PATH
ash repl --no-history
ash dot --format dot PATH
ash dot --colors --name NAME PATH
```

The `test` command supports `human` and `json` output plus synthesized-test controls. The `dot` command emits DOT by default; SVG output requires Graphviz according to help text.

## Daemon

`ash daemon` is the local RuntimeKernel daemon command family. These are subcommand names only; most daemon operations require socket, root, state, cache, log, workflow, or instance arguments described by live subcommand help:

```bash
ash daemon serve --root DIR --socket PATH --state-dir DIR --cache-dir DIR --log-dir DIR
ash daemon list --socket PATH
ash daemon start --socket PATH WORKFLOW
ash daemon start-execute --socket PATH WORKFLOW
ash daemon status --socket PATH --instance ID
ash daemon cancel --socket PATH INSTANCE_ID
ash daemon reload --socket PATH
```

This page records the command map only. Runtime authority and integrity boundaries belong to the runtime pages. Until TASK-996 expands them, use [Runtime daemon](../runtime/daemon.md) as the current detail target.

## Non-Goals

The `ash` CLI page does not define Ashgrove installation policy, remote daemon service policy, hosted registry behavior, global/system installation, OS package-manager integration, or arbitrary dependency resolution.
