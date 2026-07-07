# TASK-1948: Canonical App Template Corpus

**Status:** Planned
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

- [ ] Canonical templates exist and are indexed.
- [ ] Templates instantiate into current syntax.
- [ ] Provider/profile requirements are explicit.
- [ ] Template gates prove parse/check/run or artifact conformance.
