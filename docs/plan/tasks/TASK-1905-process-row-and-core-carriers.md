# TASK-1905: Process Row And Core Carriers

**Status:** ✅ Complete
**Phase:** [PLAN-195: Process And Concurrency Model](../PLAN-195-PROCESS-AND-CONCURRENCY-MODEL.md)

## Description

Add row, Core, and CPS carriers for process lifecycle, channel, cancellation, and ownership-transfer
facts.

## Requirements

- Preserve process facts through parser/typecheck/Core/CPS boundaries.
- Keep process rows authority-neutral.
- Fail closed on unsupported process facts or row tails.

## TDD Steps

1. Write failing row preservation and unsupported-fact tests.
2. Add minimal carriers and lowering paths.
3. Prove process rows do not install authority or handlers.

## Completion Checklist

- [x] Process row facts are represented in relevant carriers.
- [x] Unsupported process facts fail closed.
- [x] Cross-boundary preservation tests pass.
