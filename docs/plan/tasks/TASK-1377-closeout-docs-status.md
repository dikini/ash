# TASK-1377: Closeout — docs, status, CHANGELOG

## Status: ✅ Complete

## Description

Update Phase 136 status surfaces, reconcile documentation with the implemented MVP, and record final verification expectations for controller-run full workspace gates.

## Requirements

1. Create task files for all tasks (if not already done)
2. Update `PLAN-INDEX.md` with Phase 136
3. Update `CHANGELOG.md`
4. Update `DESIGN-NOTE-INTERFACE-LAWS.md` — mark implemented stages complete without overclaiming deferred work
5. Run full workspace gates, or explicitly leave them for controller verification

## Completion Notes

- Phase 136 PLAN-INDEX status reconciled to complete/implemented MVP.
- TASK-1377 PLAN-INDEX row reconciled to complete.
- Design note status changed from draft to implemented MVP, with Stage 1/2/3 sections updated to reflect the implemented local/static law, proof, synthetic test, proof-fuel/coverage/cycle, and `Prop` slices.
- Deferred boundaries preserved for attribute syntax, external prover integration, full codegen/runtime proof erasure, broad dependent types, `BoundedEquiv`, and broader effect/tower-carrier semantics.
- CHANGELOG updated for TASK-1377 closeout/status reconciliation and stale planned Phase 136 wording removed.
- Full workspace gate fallout was reconciled by updating stale stdlib parser/corpus baselines for the new `std/src/algebra/eq.ash` module and `std::io::path` law helper function.

## Verification Evidence

Phase 136 task-file existence check was run from `/home/dikini/Projects/ash`:

```bash
missing=0
for f in \
  docs/plan/tasks/TASK-1359-add-eq-interface.md \
  docs/plan/tasks/TASK-1360-parser-law-in-interfaces.md \
  docs/plan/tasks/TASK-1361-parser-law-module-scope.md \
  docs/plan/tasks/TASK-1362-parser-proof-in-impls.md \
  docs/plan/tasks/TASK-1363-parser-proof-module-scope.md \
  docs/plan/tasks/TASK-1364-typeck-law-name-checking.md \
  docs/plan/tasks/TASK-1365-typeck-proof-name-checking.md \
  docs/plan/tasks/TASK-1366-typeck-law-purity-restriction.md \
  docs/plan/tasks/TASK-1367-typeck-proof-totality-stub.md \
  docs/plan/tasks/TASK-1368-runner-law-extraction.md \
  docs/plan/tasks/TASK-1369-runner-synthetic-test-generation.md \
  docs/plan/tasks/TASK-1370-runner-by-test-delegation.md \
  docs/plan/tasks/TASK-1371-cli-law-opt-out.md \
  docs/plan/tasks/TASK-1372-law-cache-implementation.md \
  docs/plan/tasks/TASK-1373-integration-stdlib-algebra-laws.md \
  docs/plan/tasks/TASK-1374-integration-stdlib-path-law.md \
  docs/plan/tasks/TASK-1375-stage3-totality-checking.md \
  docs/plan/tasks/TASK-1376-stage3-prop-kind.md \
  docs/plan/tasks/TASK-1377-closeout-docs-status.md; do
  if [ ! -f "$f" ]; then echo "MISSING $f"; missing=1; fi
done
if [ "$missing" -eq 0 ]; then echo "All Phase 136 PLAN-INDEX task files exist."; fi
```

Result:

```text
All Phase 136 PLAN-INDEX task files exist.
```

Full PLAN-INDEX task-link existence check was also run from `/home/dikini/Projects/ash`:

```bash
python3 - <<'PY'
import re
from pathlib import Path
text = Path('docs/plan/PLAN-INDEX.md').read_text()
links = sorted(set(re.findall(r'\]\((tasks/[^)]+\.md)\)', text)))
missing = [link for link in links if not (Path('docs/plan') / link).exists()]
print(f'PLAN-INDEX task links checked: {len(links)}')
if missing:
    print('Missing task files:')
    for m in missing:
        print(m)
    raise SystemExit(1)
print('All PLAN-INDEX task links exist.')
PY
```

Result:

```text
PLAN-INDEX task links checked: 955
All PLAN-INDEX task links exist.
```

## Acceptance Criteria

- [x] All task files referenced from PLAN-INDEX exist (955 links checked), including the Phase 136 task set
- [x] PLAN-INDEX updated
- [x] CHANGELOG updated
- [x] Design note updated
- [x] Full gates pass:
  - `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo fmt --check` — passed
  - `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-cli --test stdlib_corpus_check -- --nocapture` — 2 passed after updating the stdlib corpus baseline to include `std/src/algebra/eq.ash`
  - `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test -p ash-parser --test stdlib_parsing test_io_path_public_functions_parse_as_real_fn_definitions -- --nocapture` — 1 passed after updating the path public-function baseline for `preserves_absolute_after_join`
  - `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo test --workspace` — passed
  - `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed
  - `CARGO_BUILD_RUSTC_WRAPPER= RUSTC_WRAPPER= cargo doc --workspace --no-deps` — passed

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- All TASK-1360 through TASK-1376
