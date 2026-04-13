# TASK-542: pub fn parse failure diagnostics

## Status: Complete

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

- [x] Return type changed to Result with diagnostic
- [x] Warning produced on parse failure
- [x] Valid pub fn exports unaffected
- [x] Existing tests pass

## Design Decision

`collect_module_exports` (module loading path) intentionally silences diagnostics
because it's the internal import path -- the file is already loaded and types are
being extracted. Diagnostics are surfaced only through `check_module_file`, which
is the user-facing validation path. This is a deliberate tradeoff: the loading path
must not fail on a single broken `pub fn` when other definitions are valid.

