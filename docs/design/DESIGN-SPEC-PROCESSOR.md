# DESIGN-SPEC-PROCESSOR: The Ash Spec Processor

## Status: Draft

## 1. Purpose

The spec processor is a canonical Ash workflow that audits the Ash repository for:
- spec cross-reference integrity,
- example-syntax conformance,
- PLAN-INDEX coherence,
- changelog completeness,
- and surface-syntax drift between documentation layers.

It serves three functions:
1. **Acceptance test** — proves Ash can self-host a real sysadmin/doc-tooling task.
2. **Gap discovery engine** — finds language/stdlib/spec deficiencies and emits structured findings.
3. **Breaking-change gate** — classifies findings by tier and enforces an escalation protocol for any change that risks semantic breakage.

## 2. Core design principles

### 2.1 Ground truth is `.ash` source text
Every representation of the spec processor (graph, notebook, task file) derives from the canonical source file `spec_processor.ash`.

### 2.2 The processor is meta-stable
The processor can audit its own validation rules. Any change to the processor's logic is subject to the same tiered review as compiler changes.

### 2.3 Predict forks, parallelize independent branches
When planning the processor, we must identify **high-probability breaking decisions** up front and structure work so that:
- independent tracks proceed in parallel,
- decision-dependent tracks are queued at the merge point,
- no team (or sub-agent) blocks on an unresolved Tier 2 question.

### 2.4 Safety over velocity
For Tier 2 findings, the protocol is **stop-first, optimize-path-second**. Autonomy is preserved for Tier 0 and Tier 1.

## 3. Architecture

```
┌────────────────────────────────────────────────┐
│  Input: Ash repo file tree                    │
├────────────────────────────────────────────────┤
│  Stage 1: Collect                              │
│  - Gather spec, plan, example, changelog files │
├────────────────────────────────────────────────┤
│  Stage 2: Extract                              │
│  - Parse frontmatter, headings, code blocks    │
│  - Extract task references, links, AST shapes  │
├────────────────────────────────────────────────┤
│  Stage 3: Validate                             │
│  - Run rules against extracted data            │
│  - Emit structured findings                    │
├────────────────────────────────────────────────┤
│  Stage 4: Classify                             │
│  - Assign tier (0 / 1 / 2) to each finding     │
│  - Build capability boundary delta             │
├────────────────────────────────────────────────┤
│  Stage 5: Report                               │
│  - Structured JSON / human-readable output     │
│  - Exit code reflects blocked status           │
└────────────────────────────────────────────────┘
```

## 4. Structured findings

Every gap produces a `SpecFinding`:

```ash
record SpecFinding {
    id: String,
    category: FindingCategory,
    severity: Severity,
    affected_spec: Option<String>,
    affected_plan: Option<String>,
    description: String,
    workaround: Option<String>,
    proposed_remediation: Option<String>,
    breaking_potential: BreakingPotential,
    tier: Tier,
}

enum FindingCategory {
    LanguageGap,
    StdlibGap,
    SpecDrift,
    ToolingGap,
    ExampleFailure,
    IndexIncoherence,
    ChangelogMissing,
}

enum Severity { Blocking, Degraded, Cosmetic }
enum BreakingPotential { None, Low, High }
enum Tier { T0, T1, T2 }
```

## 5. Tiered review protocol

### 5.1 Tier 0 — Autonomous

**Criteria:** Pure addition; no contract change.

- New stdlib module
- New task file
- New example/test
- CHANGELOG entry

**Action:** Proceed autonomously. Emit finding, create task, update PLAN-INDEX.

### 5.2 Tier 1 — Semi-autonomous

**Criteria:** Implementation detail change; no spec/AST/API contract change.

- Parser combinator refactor
- Adding spans to existing error variant
- `pub(crate)` → `pub` on internal helper
- Performance optimization with identical semantics

**Action:** Implement, then spawn code-review sub-agent. Merge on approval.

### 5.3 Tier 2 — Reinforced review (Red Team)

**Criteria:** Any change that could break downstream consumers, proofs, or tooling.

- AST enum variant changes
- Surface syntax changes
- Type-checker judgment changes
- Public API signature changes in `ash-core` / `ash-engine`
- `Capability`, `Effect`, or `RuntimeError` representation changes
- Editing a spec to silence a processor finding

**Action:**

```
STOP → Write DESIGN note → 24h cooling period →
Red Team debate (Advocate vs Skeptic subagents) →
Stakeholder confirmation → Proceed or Rework
```

**The "Green-is-not-truth" rule:**
If a finding has `affected_spec != NONE` and the proposed fix edits that spec, the tier is **always T2**.

## 6. Predictive fork planning

When planning the spec processor, we identify **high-probability decision points** and structure parallel tracks around them.

### 6.1 Predicted high-probability forks

| Decision | Why high-probability | Independent parallel track |
|----------|---------------------|---------------------------|
| **D1: Does `std::process` use `Capability` or a new `Process` effect?** | ~~Open~~ **RESOLVED: built-in auto-registered `Capability`** | Build `std::regex` and `std::markdown` in parallel; `std::process` integration proceeds immediately |
| **D2: Does the processor run `ash check` per-file or use a batch engine API?** | ~~Open~~ **RESOLVED: per-file subprocess** | Build the pure-string linting rules in parallel; batch engine API is a future optimization |
| **D3: Does `std::json` reuse an existing Rust JSON dependency or become a pure-Ash parser?** | ~~Open~~ **RESOLVED: hybrid** (Rust-backed parse/stringify + pure-Ash `JsonValue` AST) | Build report formatting and CLI output parsing using `JsonValue` constructors immediately |
| **D4: Should markdown parsing be a full AST or a heading/code-block extractor?** | ~~Open~~ **RESOLVED: full CommonMark AST** | Build the spec-link graph logic using heading anchors and link targets independently |

**D4 Resolution:**
- The markdown parser must produce a **full CommonMark-compliant AST**.
- The AST type must be **extension-pluggable** for future formats: GitHub Flavored Markdown, Mermaid, D2, LaTeX math.
- The AST shape should be **Pandoc-filter-compatible** where feasible: serialisable to/from a JSON representation aligned with Pandoc's native filter format (https://github.com/jgm/pandoc/blob/main/doc/customizing-pandoc.md).
- **MVP scope:** CommonMark core + JSON round-trip. Extensions are architecture-only (the AST enum has an `Extension(Block/Inline)` escape hatch) until a concrete use case demands them.

### 6.2 Parallel track diagram

```
Track A: Core processor (pure-string rules)
┌────────────────────────────────────────────────┐
│  A1: File collection and path traversal          │
│  A2: PLAN-INDEX parsing and coherence checks     │
│  A3: Spec cross-reference link validation        │
│  A4: Changelog policy checks                     │
└────────────────────────────────────────────────┘

Track B: Stdlib substrates (independent substrates)
┌────────────────────────────────────────────────┐
│  B1: std::regex interface + Rust backend         │
│  B2: std::markdown CommonMark AST + JSON filter  │
│      compatibility (extensions architected, not  │
│      implemented)                                │
│  B3: std::json interface (stub until D3 resolves) │
└────────────────────────────────────────────────┘

Decision gates (merge points)
┌────────────────────────────────────────────────┐
│  D1 → Merge std::process into processor          │
│  D2 → Merge example conformance (ash check)     │
│  D3 → Merge JSON report serialization            │
│  ~~D4 →~~ RESOLVED: full CommonMark AST with    │
│        extension-pluggable architecture           │
└────────────────────────────────────────────────┘

Track C: Integration and meta-validation
┌────────────────────────────────────────────────┐
│  C1: Processor runs against its own repo         │
│  C2: Capability boundary audit                   │
│  C3: Meta-check: processor audits its own rules  │
└────────────────────────────────────────────────┘
```

**Rule:** Tracks A and B run in parallel. Track C gates on A and B, but only after all decision gates resolve.

## 7. Capability boundary mechanism

The processor declares its expected language capabilities in `capability_boundary.ash`. The example below shows the **initial pre-Track-B state**; Track B tasks (TASK-595 through TASK-598) flip these flags from `false` to `true` as each substrate is verified.

```ash
let expected_capabilities = {
    file_io: true,
    process_spawn: false,  // becomes true after TASK-598
    regex_matching: false, // becomes true after TASK-595
    markdown_parsing: false, // becomes true after TASK-596
    json_parsing: false,   // becomes true after TASK-597
    first_class_functions: true,
    generic_interfaces: false, // still pending Phase 83
};
```

At runtime:
- If a capability is `false`, the processor skips the dependent validation and applies a pure-string workaround.
- The integration task (C2) audits this boundary and flips entries to `true` when the underlying substrate is verified.

## 8. Integration with Ash skills

- `spec-prerequisite-discovery-and-phase-scheduling` — Tier 0/1 findings are spun out as explicit prerequisite phases.
- `writing-plans` — Tier 2 findings require a DESIGN note and full plan before implementation.
- `ash-phase-implementation` — The processor phase ends with integration task C, which runs the processor and audits the boundary.
- `requesting-code-review` — Tier 1 gate.
- `subagent-driven-development` — Tier 2 Red Team debate.

## 9. Anti-patterns

| Anti-pattern | Consequence | Prevention |
|--------------|-------------|------------|
| Editing a spec to make the processor green | Spec drift, loss of normative authority | "Green-is-not-truth" rule: spec edits triggered by processor findings are always Tier 2 |
| Blocking Track A on D1 | Wasted parallelism | Predict forks in planning; queue only at merge points |
|| Silently deferring a Tier 2 finding | Technical debt accumulates, downstream phases break | The processor report must include `blocked: true` and fail CI |
|| Over-engineering markdown extensions now | Scope creep in B2 | MVP = CommonMark core + `Extension` escape hatch; extensions are architecture-only until demanded |

## 10. Relationship to other documents

- [DESIGN-VP-001-MODALITY-ONTOLOGY.md](visual-programming/DESIGN-VP-001-MODALITY-ONTOLOGY.md) — the processor is a text-primary tool with graph-secondary potential.
- `DESIGN-VP-002-REPL-NOTEBOOK.md` — planned; the processor is an ideal notebook-cell demonstration (run each validation rule as a cell).
- `docs/spec/SPEC-039` / `SPEC-042` — span and formatter fidelity are prerequisites for round-tripping processor outputs.
- [DESIGN-NOTE: Shared Document / Corpus Analysis Substrate](DESIGN-NOTE-CORPUS-ANALYSIS-SUBSTRATE.md) — the spec processor should share reusable corpus discovery, markdown/frontmatter extraction, normalized artifact identity, relationship-graph, and evidence/finding substrate with the Ash wiki while remaining a separate CI/repository-audit product rather than merging product semantics.

## 11. References

- `spec-prerequisite-discovery-and-phase-scheduling` skill
- `ash-phase-implementation` skill
- `writing-plans` skill
- `ash-task-template` skill
