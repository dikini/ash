# TASK-1948: Canonical App Template Corpus

**Status:** Complete
**Phase:** [PLAN-199: Productive App Libraries And Templates](../PLAN-199-PRODUCTIVE-APP-LIBRARIES-AND-TEMPLATES.md)

## Description

Add canonical current-syntax app templates for common productive Ash use cases.

## Requirements

- Include templates for CLI tool, file pipeline, HTTP fetch/process, supervised worker, and
  provider-profile test app.
- Each template must declare required profiles, providers, resources, and evidence expectations.
- Each template must instantiate into current target syntax and pass parse/check/run or artifact
  gates.
- Templates must not rely on legacy `workflow` syntax as target primitive.

## TDD Steps

1. Add failing conformance tests for each canonical template.
2. Add minimal template files and metadata.
3. Run instantiation and conformance checks.
4. Add negative tests for profile/provider omissions.

## Completion Checklist

- [x] Canonical templates exist and are indexed.
- [x] Templates instantiate into current syntax.
- [x] Provider/profile requirements are explicit.
- [x] Template gates prove parse/check/run or artifact conformance.

## Evidence

- Added canonical manifests under `templates/apps/` for `cli-tool`, `file-pipeline`,
  `http-fetch-process`, `supervised-worker`, and `provider-profile-test`.
- Added [templates/apps/README.md](../../../templates/apps/README.md) as the local corpus index.
- Each manifest declares required profiles, provider operations, resources, evidence expectations,
  generated files, and `ash check` conformance commands.
- Focused verification:
  `cargo test -p ash-cli --test phase199_canonical_templates -- --nocapture`.
