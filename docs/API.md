# Ash API Orientation

This page is a current orientation for repository APIs. It intentionally avoids stale code samples
that construct removed workflow/tower carriers directly.

## Current Entry Path

Productive Ash source uses target `fn main` entrypoints, explicit imports, row/profile metadata,
and checked standard library helpers. See:

- [Tutorial](TUTORIAL.md)
- [Productive Apps Tutorial](tutorials/phase199-productive-apps.md)
- [Examples](../examples/README.md)
- [App Templates](../templates/apps/README.md)

## Rust Crate Boundaries

- `ash-parser`: parses target Ash source and exposes source/surface structures used by the engine
  and tooling.
- `ash-typeck`: checks target source metadata, rows, contracts, evidence, provider/profile facts,
  and runtime-entry constraints.
- `ash-engine`: loads modules, checks entries, builds runtime artifacts, and wires standard
  profiles.
- `ash-core`: owns shared values, runtime-kernel identities, reports, provenance, contracts, and
  lower-level IR support.
- `ash-cli`: exposes `ash check`, `ash run`, `ash trace`, `ash dot`, templates, examples, and
  daemon-facing commands.
- `ash-lsp-core` and `ash-lsp`: expose parser/checker-backed language intelligence for current
  Ash syntax.
- `ash-repl`: provides interactive entry/session handling over the current parser/checker path.

For exact Rust APIs, build local rustdoc from the repository root:

```bash
cargo doc --workspace --no-deps
```

## Deprecated API Boundary

Removed Ash source forms are not valid examples for new code. Do not use legacy workflow
declarations, removed observe-with or act-with forms, or public Act/Proc/Workflow carrier
spelling in productive Ash sources, examples, templates, or fixtures. Historical design material
must be treated as prose-only context unless a Phase 201 audit explicitly classifies it otherwise.
