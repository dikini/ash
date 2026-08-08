# Ash development methods and cadence audit

**Date:** 2026-08-08

**Audited period:** 2026-07-27 through 2026-08-08, with particular attention to the
Phase 207 sessions from 2026-08-02 through 2026-08-07

**Scope:** Development sessions, planning and semantic-evidence schema, task sizing, status
reporting, agent workflow, skill routing, verification gates, and their effect on executable
target-spec delivery, language simplification, deletion work, cadence, and token use

## 1. Verdict

Ash's workflow is optimized for documenting partial layer work, not for delivering executable
language features. The project should freeze language expansion, reduce the language deliberately,
and organize all further work around small vertical conformance slices.

The principal process failure is a mismatch between what is counted as complete and what users can
execute. Phase 207 is reported as complete for its frozen route in
[PLAN-INDEX](../plan/PLAN-INDEX.md), while the subsequent
[specification-to-execution gap audit](2026-08-07-spec-to-execution-gap-audit.md) concludes that the
target language remains `partial / tested / below_spec`. Valid, parsed, and type-checked programs
such as a simple integer `match` still cannot reach checked Core.

The single most important correction is:

> A retained feature is not complete until a representative valid source program traverses the
> canonical Surface Ash → checked Core → checked CPS → Engine → terminal route, with applicable
> CLI/daemon parity and no compatibility fallback.

Task counts, commit counts, layer handoffs, status prose, and metadata transport are not substitutes
for that outcome.

## 2. Evidence reviewed

The audit reviewed:

- Codex session histories under `~/.codex/sessions/2026/`, including the Phase 202 continuation and
  the Phase 207 sessions from August 2 through August 7;
- direct user prompts in `~/.codex/history.jsonl`;
- Git history on `main` from 2026-07-27 through 2026-08-08;
- [PLAN-INDEX](../plan/PLAN-INDEX.md),
  [PLAN-203](../plan/PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md), and
  [PLAN-207](../plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md);
- [RUNNABLE-ASH-MATRIX](../plan/RUNNABLE-ASH-MATRIX.md),
  [SEMANTIC-RULE-COVERAGE](../plan/SEMANTIC-RULE-COVERAGE.md),
  `semantic-task-records.json`, and `SEMANTIC-TRACEABILITY.json`;
- the task corpus under `docs/plan/tasks/` and representative large tasks such as
  [TASK-2073](../plan/tasks/TASK-2073-checked-module-finalization-and-export-closure.md);
- project agent guidance, skills, hooks, and documentation gates; and
- the [2026-08-07 execution-gap audit](2026-08-07-spec-to-execution-gap-audit.md).

Session token figures in this report are cumulative Codex goal counters, not billing estimates.
Session archive size is evidence of context duplication, not a direct token-charge measurement.

## 3. Last-week development evidence

### 3.1 Activity did not reliably produce executable coverage

The previous calendar week produced 35 commits. The trailing 2026-08-01 through 2026-08-08 window
produced 113 commits. Across the broader audited interval there were 145 commits, about 148,900 added
lines, 81,500 deleted lines, and 2,483 file-change entries.

The high activity did not establish target-language executability. On August 7, one Phase 207
completion commit added about 18,500 lines. A cleanup later the same day removed more than 20,000
lines across 170 files. The final execution audit then found that general matches, ADT
construction, lists, closures, recursion, builtins, handlers/providers, and other specified
constructs still did not traverse the canonical route.

### 3.2 Phase-sized goals consumed excessive context

The Phase 207 goal was phrased as "implement phase 207" rather than as a bounded executable
outcome.

- The August 2 session consumed about 1.73M cumulative tracked tokens before the first abort.
- The August 3-4 continuation reached about 6.97M tracked tokens, 44 compactions, 208 recorded agent
  spawns, and 2,987 waits.
- The August 5 continuation reached about 18.34M tracked tokens and 98 compactions.
- The August 6-7 continuation reached about 24.79M tracked tokens and 170 compactions.

The user repeatedly asked what remained, why estimates had not moved, which tasks were blocked, and
whether Ash could actually execute a program. Across 115 direct messages in the trailing-week
window, 27 concerned status, progress, blockers, remaining work, or estimates.

The sessions also show significant context propagation through subagents and automatic goal
continuations. The requested date directories contain several gigabytes of session archives, much
of it duplicated history.

### 3.3 Scope changes were absorbed into one long-running goal

Role and policy work moved through several positions inside the same Phase 207 goal:

1. implementation subject;
2. minimal stubs;
3. non-authorizing metadata;
4. exclusion from completion criteria; and
5. deletion without compatibility.

Dynamic loading underwent a similar late exclusion. These were legitimate product decisions, but
continuing the same phase goal caused plans, tests, evidence records, and estimates to be repeatedly
reconciled instead of closing the old scope and starting a new bounded task.

## 4. Project-management schema findings

### 4.1 The planning corpus is too large for routine orientation

At the audited revision:

- `docs/plan/` contained 1,882 files;
- `docs/plan/tasks/` contained 1,645 task files;
- there were 133 `PLAN-*` files, 65 plan audit files, 127 spec files, and 52 note files;
- `PLAN-INDEX.md` contained 1,636 lines;
- `PLAN-INDEX-HISTORY.md` contained 4,081 lines;
- `SEMANTIC-RULE-COVERAGE.md` contained 2,274 lines;
- `semantic-task-records.json` contained 2,175 lines;
- `SEMANTIC-TRACEABILITY.json` contained 2,381 lines; and
- `CHANGELOG.md` contained 5,932 lines.

Historical material dominates the default read path. An agent attempting to determine current work
must distinguish active, historical, partial, completed-handoff, below-spec, deferred, and removed
states across multiple files before it can begin the actual task.

### 4.2 Status is duplicated and contradictory

The same task status and evidence are copied among PLAN-INDEX, a phase plan, a task file,
SEMANTIC-RULE-COVERAGE, semantic-task-records JSON, traceability JSON, audits, and changelog entries.
For example, `TASK-2073` appears hundreds of times in the central semantic ledgers alone.

This duplication allows PLAN-INDEX to report Phase 207 tasks as complete while semantic coverage
still carries stale in-progress and below-spec descriptions. The
[RUNNABLE-ASH-MATRIX](../plan/RUNNABLE-ASH-MATRIX.md), which should be the most useful current
orientation artifact, was last verified on 2026-07-28 and remained a manually maintained task
ledger.

### 4.3 Administrative write amplification is extreme

During the trailing August 1-8 window of 113 commits:

- 91 commits touched `CHANGELOG.md`;
- 78 touched `SEMANTIC-RULE-COVERAGE.md`;
- 70 touched `semantic-task-records.json`;
- 69 touched `SEMANTIC-TRACEABILITY.json`;
- 53 touched `PLAN-207-COMPLETE-MODULE-REALIZATION.md`; and
- 46 touched `PLAN-INDEX.md`.

The workflow therefore creates several status edits for many implementation edits. Those edits then
require their own tests, repairs, and reconciliation commits.

### 4.4 Tasks are horizontal handoffs rather than executable slices

Layer ownership is architecturally useful, but the current schema allows a parser, Type, Core, CPS,
admission, or client handoff to be marked complete independently. A whole phase can consequently
contain completed handoffs while the representative source construct still does not execute.

TASK-2073 demonstrates the size problem: its task file changed in 43 commits, its main
implementation file in 37, and its principal test file in 30. It contained multiple independent
namespace, dependency, visibility, type, and finalization obligations and should have been split
before implementation.

## 5. Recommended operating model

### 5.1 Stop adding numbered phases

After Phase 207, use one recovery programme with three outcome milestones:

1. **Minimal executable Ash**
   - reconcile the target specification;
   - classify every construct as `keep-now`, `delete`, or `defer`; and
   - complete ordinary first-order functions, data construction, patterns, matches, static modules,
     imports, records, and required builtins.
2. **Executable effects**
   - general operation calls;
   - providers and handlers; and
   - one general checked admission seam with no exact-source special cases.
3. **Optional advanced language**
   - closures, higher-order functions, evaluation modes, contracts, proofs, comprehensions,
     macros, or reflection only if they survive the language census.

Only one milestone and one implementation slice should be active at a time.

### 5.2 Make tasks vertical

Every semantic implementation task should deliver one vertical outcome:

```text
canonical rule
→ source program
→ parse/type
→ checked Core
→ checked CPS
→ admission
→ Engine
→ CLI and daemon result
```

A well-sized task has:

- one canonical semantic rule or one removal decision;
- one user-visible executable or rejection outcome;
- one positive acceptance case;
- relevant negative and mutation cases;
- CLI/daemon parity where applicable;
- one working day as the target; and
- roughly one to three coherent commits.

Split before implementation when the task contains independent constructs, is expected to exceed
one working day, or survives two context compactions.

Do not use a compound status such as `Complete — partial/tested/below-spec`. A task is done only
against its precise acceptance criteria. Broader rule coverage is a separate, derived measurement.

### 5.3 Establish one writable source of status

Replace manually synchronized ledgers with one small machine-readable active record:

```yaml
id:
outcome:
rule:
decision: keep-now | delete | defer
state: ready | active | blocked | done | dropped
acceptance_tests:
blocked_by:
next_action:
verified_commit:
token_budget:
```

Generate the roadmap, runnable-language matrix, spec coverage, task index, weekly status, and CI
dashboard from this record and test metadata. Archive historical phase plans and task files as
read-only history outside the default agent read path.

### 5.4 Make status self-service

Add one command, such as `cargo xtask status`, that reports:

```text
active outcome
acceptance cases passed/total
current failing boundary
blockers requiring a decision
exact next action
last verified commit
retained/deleted/deferred construct counts
remaining compatibility routes
elapsed and token budget
```

Update the underlying record at each completed slice. Status becomes observable repository state,
not a report that the user must repeatedly request.

## 6. Language simplification policy

Before further implementation, create a complete surface census. For every grammar construct choose
exactly one disposition:

- **Keep now:** make it executable through the canonical route.
- **Delete:** remove grammar, AST, type checking, lowering, runtime code, documentation, and tests in
  the same slice.
- **Defer:** remove it from the current target grammar and place it in a short design backlog.

Deferred syntax should not continue parsing as if supported. No syntax should remain in the target
merely because a parser or legacy evaluator once accepted it.

Immediate deletion or deferral candidates include:

- exact-source compatibility admission routes;
- parser/typechecker forms with no intended Core projection;
- comprehensions, typed `do`, reflection/reification, and multi-shot extensions until their
  specifications are reconciled;
- legacy task-specific test APIs;
- remaining role, policy, or dynamic-loading residue; and
- metadata-only features not required by retained executable programs.

The shortest implementation order, consistent with the execution-gap audit, is:

1. reconcile the executable target;
2. complete values, constructors, patterns, and matches;
3. complete first-order calls and required builtins;
4. add general handler/provider execution;
5. add higher-order functions and modes only if retained;
6. add contracts, evidence, traces, and monitors only if retained; and
7. delete route exceptions and establish generated conformance.

## 7. Skills and agent workflow

### 7.1 Reduce the skill catalog

The audited workspace advertised roughly 70 skills. Fifty global `~/.agents/skills` entries were
unrelated marketing skills, and two identical copies of `rust-skills` were discoverable. This adds
routing context and increases the chance of irrelevant skill activation.

Keep about five Ash-facing skills:

- `ash-work`: orientation, slicing, and the active-task schema;
- `ash-semantics`: target authority and vertical realization;
- `rust-dev`: concise Rust rules plus TDD;
- `ash-debug`: systematic diagnosis; and
- `ash-verify`: closeout and full gates.

Remove unrelated skills from this workspace's discovery path, keep one Rust skill, and consolidate
overlapping TDD, planning, subagent, review, and verification instructions. Keep each main skill
short and load specialized references only when triggered. Prefer checked scripts to long prose
procedures.

The current project TDD skill duplicates AGENTS.md, contains malformed text, references the wrong
`docs/specs/` path, and mandates the same four-agent ceremony. It should be replaced rather than
expanded.

### 7.2 Replace ritual subagent roles with bounded collaboration

The mandatory test-agent → code-agent → QA-agent → review-agent chain produces context duplication,
waits, and artificial handoffs between work that belongs to one TDD loop.

Use instead:

- one implementer owning RED → GREEN → REFACTOR;
- one independent reviewer for risky or substantial slices;
- CI for routine QA;
- subagents only for genuinely independent work;
- at most two subagents per slice;
- `fork_turns="none"` with a 1-2 KB task capsule by default;
- one root session per task; and
- a mandatory split or checkpoint after two compactions or when the agreed budget is exceeded.

## 8. Verification and changelog policy

Use three verification levels:

1. **Iteration:** formatting, affected-crate check, and focused failing/passing tests.
2. **Slice close:** affected-crate tests, clippy, documentation/schema validation, and the vertical
   acceptance corpus.
3. **Push/CI:** full workspace suite, full documentation gate, security checks, and fuzz checks where
   applicable.

Update `CHANGELOG.md` once per completed user-visible slice, not for intermediate test wording,
ledger repairs, or every implementation commit.

Require property tests when an algebraic or generated invariant exists. Do not require them for
every documentation, wiring, deletion, or single-path integration task.

## 9. Test cleanup policy

Classify every test as one of:

- `conformance`: protects retained target semantics;
- `compiler-unit`: protects an implementation invariant;
- `removal-fence`: proves deleted syntax or routes stay absent; or
- `legacy`: delete.

Organize semantic tests by rule or language feature rather than historical task number. Generated
conformance cases should become the evidence ledger. Task prose should not duplicate test counts and
evidence descriptions across several documents.

## 10. Recommended first actions

Before more compiler work:

1. Freeze new language features.
2. Create the keep/delete/defer construct census.
3. Select the minimal executable milestone.
4. Replace writable planning ledgers with one active record and generated status.
5. Rewrite AGENTS.md and reduce the skill catalog.
6. Reclassify tests and identify immediate legacy deletions.
7. Start the first vertical slice with the integer-pattern counterexample from the execution-gap
   audit.
8. Follow with an ADT construction/match slice and a cross-module ADT slice.

## 11. Outcome metrics

Track outcomes rather than activity:

- retained constructs that execute through both canonical clients;
- constructs classified as retained, deleted, or deferred;
- remaining compatibility routes;
- code and tests deleted;
- median vertical-slice cycle time;
- tokens per completed vertical slice; and
- number of user decisions currently blocking progress.

Do not use phase count, task count, commit volume, documentation volume, agent count, or raw test
count as primary progress measures.

## 12. Conclusion

Ash does not primarily need more planning detail. It needs a smaller executable target, one
observable source of truth, and short tasks that prove a retained construct works through the whole
production route.

Reducing administrative duplication and agent ceremony is not merely a token optimization. It is
necessary to make completion claims correspond to executable behavior and to prevent another large
phase from accumulating millions of tokens while the central source-to-execution gap remains open.
