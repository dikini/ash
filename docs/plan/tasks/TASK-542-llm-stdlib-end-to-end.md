# TASK-542: pub fn parse failure diagnostics

## Status: Draft (v2)

## Description

Change `parse_supported_pub_fn_callable` from silent `Option` return to `Result`, producing a diagnostic warning when a `pub fn` snippet fails to parse. Prevents silent export dropping.

## Spec Reference

- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §5.3
- [DESIGN-026](../../design/DESIGN-026-MODULE-TYPE-RESOLUTION-REMEDIATION.md) D4

## Requirements

1. Malformed `pub fn` produces warning diagnostic, not silent None.
2. Valid `pub fn` still exports correctly.
3. Warning includes function name (if extractable) and reason.

## Completion Checklist

- [ ] Return type changed to Result with diagnostic
- [ ] Warning produced on parse failure
- [ ] Valid pub fn exports unaffected
- [ ] Existing tests pass

