# AUDIT-201: Deprecated Functionality Removal

**Status:** Active
**Owner:** TASK-1961
**Phase:** [PLAN-201: Deprecated Functionality Removal](../PLAN-201-DEPRECATED-FUNCTIONALITY-REMOVAL.md)

## Scope

Phase 201 removes deprecated Ash functionality from repository code. Ash source in the project
repository must use target Ash only. Deprecated Ash forms must not remain in `.ash` files,
examples, templates, fixtures, snapshots, or Rust source/test string literals.

Historical and reference documents may mention removed forms only as labeled prose. They must not
retain removed forms as Ash code blocks, examples, templates, or executable fixtures.

## Removal Outcomes

| Outcome | Meaning |
|---------|---------|
| Remove | Delete deprecated Ash code, fixtures, snippets, acceptors, or behavior. |
| Rename | Move internal or diagnostic vocabulary to target Ash terms without retaining old Ash source forms. |
| Historical prose only | Keep labeled prose discussion only; no deprecated Ash snippets or code blocks. |

## Initial Dependency Classes

| Class | Owner | Required outcome | Gate |
|-------|-------|------------------|------|
| Deprecated Ash snippets in `.ash` examples and workflow fixtures | TASK-1961, TASK-1965, TASK-1967 | Remove or rewrite to target Ash | `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate` |
| Deprecated Ash snippets in Rust tests and source string literals | TASK-1961, TASK-1962, TASK-1965, TASK-1967 | Remove or replace with token/structure tests that do not embed stale Ash | `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate` |
| Deprecated Ash snippets in markdown docs | TASK-1961, TASK-1966, TASK-1967 | Remove, rewrite, or convert to labeled prose with no Ash code block/snippet | `cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate` |
| Legacy parser/checker acceptors | TASK-1962 | Remove acceptance of removed Ash forms | focused parser/checker tests plus Phase 201 gate |
| Legacy surface AST and lowering carriers | TASK-1963 | Remove or rename to target vocabulary | focused parser/engine/Core/CPS tests plus Phase 201 gate |
| Deprecated type/effect/runtime vocabulary | TASK-1964 | Remove or rename to target vocabulary | focused type/effect/runtime tests plus Phase 201 gate |
| Formatter, LSP, template, and CLI behavior | TASK-1965 | Remove deprecated behavior and stale fixtures | focused formatter/LSP/template/CLI tests plus Phase 201 gate |
| Historical/reference docs | TASK-1966 | Prose-only references with explicit historical labels | docs gate plus Phase 201 gate |

## Current Gate

`crates/ash-cli/tests/phase201_deprecated_functionality_removal_gate.rs` is the active inventory
gate. It scans repository Ash-bearing paths for removed forms and fails on any occurrence outside
explicitly excluded historical planning/changelog/archive roots.

The gate is intentionally stricter than Phase 200 gates:

- no compatibility-only Ash snippets;
- no reference-only `.ash` files with removed forms;
- no Rust source string literals containing removed Ash snippets;
- no markdown Ash code blocks containing removed forms;
- no template or snapshot occurrences.
- no source-shaped removed Ash declarations inside multi-line Rust raw string fixture bodies.

## Current Findings

The active source-shaped Ash gate is green as of the latest TASK-1967 slice:

```bash
cargo test -p ash-cli --test phase201_deprecated_functionality_removal_gate -- --nocapture
```

The latest cleanup slices removed parser acceptance for removed `capability interface` and
`capability impl` forms, added split-token parser/engine rejection tests for those forms, tightened
Rust raw-string scanning, converted newly exposed source-shaped fixtures to target `fn`/`interface`
syntax, deleted workflow-dependent typechecker/capability-configuration tests that only exercised
removed syntax, removed the unparseable LLM router draft from the active stdlib corpus, and removed
the parser surface/typechecker registration carriers for deleted capability definition forms.

Remaining ownership is no longer source-shaped Ash snippets in active artifacts; it is internal
carrier and vocabulary removal or renaming:

- internal `Workflow`, `Act`, and `Proc` Rust carriers still need audit-driven removal or target
  vocabulary replacement where target Ash no longer depends on them;
- the role inclusion checker has been retargeted away from `WorkflowDef`; remaining workflow
  carrier ownership is concentrated in Core/runtime lowering, historical process substrates, and
  compatibility-adjacent tests that must be audited before removal;
- type/effect/runtime and report terminology still contains capability implementation/interface
  vocabulary that must be classified as target runtime/provider vocabulary, renamed, or removed;
- LSP current-symbol paths no longer expose removed workflow entries or removed capability
  definition forms as completions, document/workspace symbols, goto targets, hover payloads, or
  db symbol-index entries;
- `ash-lint` no longer walks removed `module.workflow` declarations, no longer carries the
  workflow-specific L004 rule implementation, and no longer constructs removed workflow surface
  carriers in active lint tests;
- REPL completion/session vocabulary has been retargeted away from removed workflow/capability
  user-facing forms; remaining CLI/daemon/report vocabulary must still be audited separately;
- CLI help/check/run/trace/dot wording has been retargeted to source/entry terminology, and
  `ash dot` now uses the current engine parser instead of the removed workflow-definition parser;
  `ash check` module-file fallback now describes old `workflow` declarations as removed syntax
  rather than current workflow keywords; daemon command help and non-schema diagnostics now use
  entry instance/definition wording, while daemon/report JSON fields and failure-class strings
  remain a separate runtime compatibility audit item;
- daemon integration fixtures now use target entry syntax for control-plane/artifact checks, and
  the stale process-carrier daemon child-failure fixture was removed from active test code;
- daemon indexing now discovers target `fn main` entries and validates them through the current
  engine/artifact path instead of relying only on removed workflow declaration slots, and daemon
  artifact requests now share the `ash run` application-entry identity shape;
- active `std/src/runtime/supervisor.ash` now uses target imports and `capability Args` syntax;
- the Phase 201 gate now rejects source-shaped removed type-carrier signatures in Rust fixture
  continuation lines, and the exposed std carrier/law-purity fixtures were removed from active
  tests;
- `ash-engine` module-resolution fixtures now use target expression bodies and parse/check
  assertions for non-application return types instead of old `return` bodies or arbitrary runtime
  entry execution;
- productive top-level docs now route through target entry examples and current API orientation,
  `docs/book/appendix-b.md` has been reduced from source-shaped stale provider examples to
  historical prose, and the Phase 201 gate scans productive docs/book/tutorial roots;
- the unadvertised `ash-fuzz` typechecker target was removed because it generated deprecated
  workflow carrier values directly and was not part of the documented current fuzz suite;
- parser/CLI/LSP removed callable-arrow diagnostics now use neutral removed-arrow wording instead
  of Act/Proc/Workflow callable messages;
- historical callable type spellings are removed from active parser acceptance:
  `Fn(<params>) -> <return>` and bare unary `<type> -> <return>` no longer parse as current Ash
  callable types; active fixtures use target `(<params>) -> <return>` syntax, and the Phase 201
  gate blocks source-shaped `Fn(...)` callable snippets. Parser surface display now renders
  `Type::Fn` with target parenthesized callable syntax instead of emitting the removed `Fn(...)`
  spelling;
- synthesized algebra-law tooling no longer models removed tower forms as deferred carrier
  variants; generated law profiles enumerate only target carriers and the removed carrier spellings
  are absent from active law-profile source literals;
- synthesized policy, obligation, and small-world fallback rows no longer describe active deferred
  target-runner behavior as legacy or compatibility behavior;
- synthesized contract-test unsupported target metadata now uses runtime-callable wording instead
  of workflow-callable wording;
- the stale Phase 98 cross-layer conformance test was removed because it depended on deleted
  workflow examples and asserted legacy workflow execution success;
- `ash-engine` no longer keeps an ordinary type-snippet compatibility parser path for active
  stdlib tests; LLM stdlib type metadata checks now parse target `ModuleFile` surfaces and lower
  through current metadata APIs;
- formatter and docs-current-syntax diagnostics now describe removed syntax rather than
  deprecated syntax in active tooling paths;
- parser fn-contract lowering no longer labels classified contract sidecars or deferred discharge
  records as legacy Stage-1 contract behavior; active discharge metadata now uses current
  classified-contract vocabulary;
- parser capability-import metadata no longer classifies current provider/action target bindings
  as legacy capability exports; active import bindings now use provider-operation vocabulary;
- parser `Decide` else-branch lowering rejection no longer describes removed else-branch carriers
  as legacy; the active diagnostic now uses removed-form vocabulary;
- callable-syntax stdlib/reference gates and agent-facing function reference prose now describe
  historical removed callable syntax rather than legacy compatibility syntax;
- old-form act block statements no longer parse as source syntax or typecheck through a
  compatibility helper; the Phase 201 gate rejects source-shaped `act { ... ret ...; }` snippets,
  while target act do-sugar remains the admitted path;
- internal `ActBlock`/`ActStmt` surface carriers and their lowering/display/lint/typecheck
  visitor branches have been removed from active parser, typechecker, engine, interp, lint, and
  REPL code. Runtime tests that still need an Act value now use current core Act closures or
  target Act do-sugar paths rather than removed surface carriers;
- the active stdlib OODA compatibility helper module and ash-lint OODA compatibility aliases have
  been removed from executable/tooling code. The Phase 201 gate now blocks reintroducing the
  stdlib OODA module/export or ash-lint OODA alias/category documentation. Productive book index
  and appendix labels were retargeted to target effects and policies, and the stale book summary
  link map was replaced with a current orientation page;
- Core text no longer admits the row/effect aliases `cap`, `op`, or `proc` in active parser paths.
  Canonical Core text now uses target `operation` and `process` spelling only, active `.core`
  fixtures were retargeted, and the Phase 201 gate scans `.core` artifacts for stale aliases;
- engine row-admission diagnostics no longer describe contract row items as legacy; unsupported
  contract row requirements now use current contract-discharge record wording;
- typechecker interface-evidence lowering helpers no longer describe current evidence argument
  conversion as legacy type conversion; active helper names now use current interface-evidence
  type vocabulary;
- the stale active TASK-1023 tower-algebra evidence integration test was removed because it still
  asserted Act/Proc/Workflow carrier evidence over current target stdlib algebra modules;
- workflow header compatibility no longer admits old authority/resource clauses in active parser
  paths: `plays role`, direct `capabilities:`, `owns`, and `uses` are removed from workflow header
  parsing, implicit-role synthesis from direct workflow capabilities was disabled, stale
  implicit-role/resource-binding parser fixtures were deleted or retargeted, and the Phase 201 gate
  blocks source-shaped reintroduction in active Ash artifacts and Rust string literals. The
  remaining source-ordered `WorkflowHeaderEvent` carrier now preserves only current
  `requires:`/`ensures:` contract clauses, and the dead `LoweredWorkflow` implicit-role wrapper was
  removed. `WorkflowDef` no longer carries direct owned-resource or used-binding header fields,
  and the typechecker no longer validates removed `owns`/`uses` header carriers constructed
  directly in Rust;
- runtime resource admission has been retargeted away from removed workflow-header ownership
  vocabulary: active APIs/tests now use entry-owned resource admission names while preserving
  existing runtime resource behavior, and runtime provenance notes now use `resource source` /
  `binding source` wording instead of source-shaped removed declaration prefixes;
- legacy module-graph crate membership aliases have been removed from active core APIs; parser
  tests and import-resolution tests use the current crate-membership API directly;
- the legacy interpreter `execute_workflow` wrapper without explicit `BehaviourContext` has been
  removed from active runtime APIs and tests now use the current behavior-context execution entry;
- provider authoring compatibility shims have been removed from active core/runtime/engine paths:
  providers without explicit operation metadata fail closed, runtime host-binding admission no
  longer bypasses row validation for shim metadata, and custom-provider tests declare target rows
  explicitly;
- dotted qualified-name compatibility parsing has been removed from the typechecker:
  `QualifiedName::parse` now rejects `.` separators and accepts only target `::` separators for
  module-qualified names;
- `check_pattern` generic ADT fallback naming has been retargeted to registered-variant
  terminology, so the active target pattern path no longer carries legacy-labeled code;
- interpreter terminal-observed execution no longer creates ambient provider contexts when
  RuntimeKernel admission is empty; the path now fails closed through explicit admitted binding id
  projection, and active runtime admission tests use authored test provider metadata;
- interpreter builtin fallback dispatch no longer labels current pattern-matched builtin handling
  as legacy; the Phase 201 gate blocks reintroducing that stale active-code label;
- typechecker do-target dictionary comments no longer label the current built-in computation
  dictionary bridge as legacy fallback behavior; the Phase 201 gate blocks reintroducing that
  stale active-code label;
- typechecker compiler-known contract intrinsic storage no longer carries workflow-intrinsic
  implementation vocabulary. Active APIs use `ContractIntrinsic*`, `contract_intrinsics`,
  `lookup_contract_intrinsic`, and `__contract_intrinsic_context`, and the Phase 201 gate blocks
  reintroducing the old carrier names in active typechecker paths;
- the obsolete `ash-typeck::capability_check` workflow-surface verifier has been removed from
  active exports and source. Stale tests that constructed removed `surface::Workflow` values via
  `CapabilityChecker`, and obsolete interpreter tests that entered runtime coverage through
  `parse_workflow::workflow_def`, `lower_workflow`, or `SurfaceWorkflow`, were removed; the Phase
  201 gate blocks reintroducing those active-code paths. Par-removal regression labels no longer
  refer to deleted `SurfaceWorkflow::Par` or capability-checker paths;
- runtime/typechecker capability requirement carriers no longer use workflow-capability naming.
  Active obligation/runtime verification APIs use `EntryCapabilities` and aggregate inputs use
  `entry_capabilities`; the Phase 201 gate blocks reintroducing the old carrier names in those
  paths;
- current provider-backed `llm::dispatch` stdlib builtin declarations now have honest interpreter
  dispatch-table entries marked unimplemented, keeping target stdlib declarations visible while
  failing closed instead of falling through as unknown functions;
- import-visibility semantic-summary tests no longer label current imported type-definition or
  semantic-summary transport as legacy TypeDef fallback behavior, and the active temp Ash
  fixtures in that suite now use target expression-tail entry bodies instead of old `return`
  statements;
- parser proposition where-bound tests no longer describe current impl where-bound preservation or
  malformed-body diagnostics as legacy. The Phase 201 gate blocks those stale labels in active
  parser proposition tests;
- parser removed-capability rejection tests no longer describe deleted capability declaration
  syntax or role-authority capability metadata as legacy. The Phase 201 gate blocks those stale
  labels in active parser module/lib tests;
- TASK-826 TypeEnv forcing-point tests no longer describe current inference-meta and deferred
  noncanonical-shape fallback behavior as legacy. The Phase 201 gate blocks those stale labels in
  the forcing-point rollout tests;
- normalizer definitional-equality documentation no longer describes the current inference-meta
  boundary as owned by a legacy unifier. The Phase 201 gate blocks that stale active API-doc label;
- typechecker semantic-summary rejection tests no longer describe malformed or unsupported
  imported computation summaries as legacy summaries. The Phase 201 gate blocks those stale labels
  in active summary tests;
- TASK-876 proposition-solver tests no longer describe forbidden no-inversion/no-mutation
  evidence facts as legacy unification/substitution/meta facts. The Phase 201 gate blocks that
  stale active test assertion label;
- alpha visible-computation acceptance tests no longer label non-interference coverage as legacy
  surfaces. The Phase 201 gate blocks that stale active test name;
- interpreter list-helper documentation no longer describes current Cons/Nil list values through
  legacy list-variant removal or transition wording. The Phase 201 gate blocks that stale active
  runtime-doc label;
- the active `WorkflowContract` source-contract carrier no longer exposes the stale
  `legacy_contract` public field name. Workflow-form lowering initializes `source_contract`, and
  the Phase 201 gate blocks the removed carrier field name;
- core public computation summary schema tests no longer label older-summary defaulting payloads
  as legacy payloads. The Phase 201 gate blocks that stale active schema-test wording;
- parser generated-identifier hygiene tests no longer label source names that resemble generated
  helper placeholders as legacy helpers. The Phase 201 gate blocks that stale active test name;
- core proposition summary schema tests no longer label pre-V5 proposition payload rejection as
  legacy payloads, facts, registration, or summary versions. The Phase 201 gate blocks those stale
  active schema-test labels;
- Type IR normal-form comments no longer describe current imported pre-attribution carriers as
  legacy carriers, and parser process-row tests name removed proc syntax directly. The Phase 201
  gate blocks those stale active labels;
- runtime actor protocol and older summary-version rejection tests no longer use legacy actor,
  capability, or module fixture identifiers. The Phase 201 gate blocks those stale active fixture
  labels;
- parser/interpreter current-wording assertions and engine import-summary tests no longer carry
  legacy vocabulary in active labels. The Phase 201 gate blocks those stale active assertion/test
  names;
- `ash.lock` source/git validation no longer labels the redundant git field as legacy in active
  engine code or registry metadata tests. The Phase 201 gate blocks those stale lockfile labels;
- active LLM provider tests no longer suppress deprecated external field access. Chat response
  fixtures decode current provider-shaped JSON, stream chunk fixtures use current defaulted fields,
  and the Phase 201 gate blocks deprecated-field suppressions in those paths;
- ashgrove active manifest, source archive, lockfile, and payload-ignore tests no longer use
  legacy labels for `.ash.toml`, `.source-rev`, redundant git metadata, or ignored sentinels. The
  Phase 201 gate blocks those stale active labels;
- productive stdlib root/runtime/LLM comments no longer describe target helper modules and entry
  execution through workflow-era wording. The Phase 201 gate blocks those stale comments;
- Phase 199/200 executable inventory tests no longer use legacy/deprecated labels for removed
  syntax inventory, and LSP document-symbol construction no longer touches the deprecated protocol
  field literal. The Phase 201 gate blocks those stale executable-crate labels;
- stale book appendices no longer present the deleted workflow-era example catalog, removed
  example tree, or removed run commands as current documentation. Appendix A now lists only the
  current checked target examples, Appendix C lists current productive docs/example roots, and the
  Core text reference uses canonical `operation`/`process` spelling;
- focused reference pages no longer present deleted Act/Proc/Workflow stdlib files, old phase
  example paths, source-shaped tower carrier snippets, or old Core alias spellings as current
  reference evidence. The affected stdlib/language/example/status/agent-card pages are now
  historical prose or route to current target examples;
- the stale Phase 199 current-syntax inventory is now explicitly superseded because its old table
  classified removed example and stdlib tower paths as current executable assets;
- full reference validation now passes after removing deleted Result/runtime/test evidence paths,
  replacing stale CPS interpreter module paths, adding missing IR leaf-page metadata, and deleting
  broken current links back into tower-era reference pages;
- remaining docs/reference source-shaped examples exposed by the stricter sweep were retargeted:
  algebra and ash-test examples now use target `pub fn main`, the legacy proof spelling was
  removed from an Ash code block, and the historical Phase 101 capability/resource parser
  substrate page no longer contains removed-form Ash code;
- target-grammar and WorkflowForm-era routing docs were retargeted for Phase 201: SPEC-095b now
  presents removed workflow/act/tower source families as historical migration-map labels rather
  than compatibility aliases and no longer includes the source-shaped workflow declaration
  example; SPEC-INDEX and NOTE-INDEX route PLAN-196 through the Phase 201 removed-form boundary;
  SPEC-056 and NOTE-010 now describe warning/translation behavior as historical migration context
  rather than current support;
- residual spec/note migration wording was retargeted so old callable, act/tower, capability, and
  workflow forms are described as removed or historical rather than compatibility syntax across
  SPEC-027, SPEC-031, SPEC-047, SPEC-052, SPEC-054, SPEC-056, SPEC-063, SPEC-072, SPEC-095b,
  SPEC-096b, SPEC-097, SPEC-097b, SPEC-098c, NOTE-010, NOTE-019, NOTE-035, and the spec README;
- remaining executable `.ash` fixtures that used `workflow main ... { ret ... }` or
  `workflow main { done }` were rewritten to target `fn main` entries, and the strict non-comment
  `.ash` source sweep for removed workflow/act/tower/capability/callable forms is silent;
- root and historical language docs are now part of the Phase 201 active-source gate:
  `README.md` was retargeted to checked target examples, `docs/SHARO_CORE_LANGUAGE.md` no longer
  carries old source-shaped scenario snippets, and active lint/stdlib comments no longer describe
  removed paths as compatibility behavior.
- `ash-engine` entry verification no longer presents the active entry contract as an entry
  workflow in helper names, diagnostics, and integration fixture prose. The remaining active
  entry tests use target `fn main`, `capability Args`, and explicit `Ok`/`Err` result bodies
  rather than stale workflow-era fixture shapes.
- LSP macro-summary identity formatting no longer emits the removed `Fn(...)` callable spelling
  for surface function types; compact summaries now use target `(<params>) -> <return>` strings.
- typechecker provider validation no longer keeps an empty-registry compatibility fallback:
  explicit `provider:action` targets require a registered provider, and direct-action tests now
  declare provider dependencies explicitly.
- Core operation rows no longer use a capability-named storage carrier: active Core APIs now use
  `CoreRowItem::Operation` and `CorePublicRowItemSummary::Operation` for operation requirements.
- Core raised operation effects no longer use a capability-named carrier: active Core APIs now use
  `CoreEffectOp::Operation` for operation effects.
- CPS resume-row validation no longer describes the active affine inherited-target-row path as
  legacy compatibility; multi-shot rejection diagnostics use current inherited-row terminology.
- active stdlib runtime supervisor comments use entry-definition wording rather than entry
  workflow wording.
- active parser, engine, and CLI paths no longer use `entry workflow` as the current entry label;
  the Phase 201 gate blocks reintroducing that stale label in those paths.
- active CLI entry-source tests no longer use workflow-named test artifacts, local variables, or
  assertions for target `ash run` entry sources; the Phase 201 gate blocks reintroducing those
  labels in the CLI entry-source tests.
- broader active CLI run/trace/admission/runtime-kernel/lexical-scope tests no longer use
  workflow-file/source labels for current target entry fixtures; the Phase 201 gate blocks
  reintroducing those stale labels in the selected CLI test paths.
- `ash-engine` module-file warning documentation no longer labels current non-fatal public
  function export diagnostics as legacy `pub fn` snippet diagnostics; the Phase 201 gate blocks
  that stale active label.
- The outstanding full `module_file_check_tests` failures from the engine warning cleanup were
  closed: deleted `std/src/act.ash` evidence was replaced with current `std/src/process.ash`,
  inline module declarations now assert the current authoritative parse failure, and balanced
  one-line imports without semicolons no longer cause the metadata stripper to hide following
  public interface definitions before constraint visibility validation.
- RuntimeKernel synthetic artifact reports no longer present the active application entry as a
  workflow artifact: synthetic TCIR now uses `ApplicationEntry` /
  `RuntimeKernel<ApplicationEntry>`, and daemon/one-shot artifact summaries compare the checked
  application-entry boundary scope.
- Active stdlib algebra interface signatures no longer use stale bare callable type syntax in
  method signatures. The current stdlib uses target callable forms such as `(A) -> B`, and module
  metadata stripping now handles balanced multi-line imports without semicolons so daemon entry
  indexing does not depend on stale parser tolerance.
- Role runtime no longer depends on deprecated workflow definition carriers for role/capability
  resolution. `RoleRegistry` resolves explicit role references plus admitted capability
  declarations directly, and the Phase 201 gate blocks reintroducing `WorkflowDef` in role-runtime
  implementation and integration-test paths.
- RuntimeKernel identity carriers no longer use workflow-named active APIs. Definition, artifact,
  instance, process-tree, and artifact-builder identity carriers now use application/entry
  vocabulary, and the Phase 201 gate blocks reintroducing the old RuntimeKernel workflow identity
  carrier names.
- Lower runtime admission and boundary carriers no longer use workflow-named active APIs.
  Admission request/outcome/requirement carriers, admission context, structured contract evidence,
  boundary outcomes, reports, failures, and engine admitted-boundary wrappers now use
  application-boundary vocabulary. The Phase 201 gate blocks reintroducing the old workflow-named
  boundary carrier APIs.
- `ash-typeck::do_target` no longer carries hard-coded Act/Proc/Workflow tower fallback support.
  Named computation constructors must resolve through explicit `Monad` evidence; hidden Act
  bind/return carriers, the tower dictionary resolver, tower-specific target diagnostics, and
  hard-coded Act/Proc/Workflow intrinsic shim strings are blocked by the Phase 201 gate.
- Runtime/Core failure attribution no longer carries active tower vocabulary. Operational failure
  attribution uses `FailureBoundary` and `boundary`, TCIR/AMIR computation provenance uses
  boundary-level and cross-boundary lift fields, daemon reports serialize boundary/application
  failure labels, and the public typechecker algebra manifest/test surface uses computation
  terminology. The Phase 201 gate blocks reintroducing the old runtime/Core tower carrier names.
- Daemon execution report vocabulary no longer uses workflow success/request/failure labels in
  active CLI report paths. Start-execute status classes and admitted-source-drift failures now use
  application/entry wording, and the Phase 201 gate blocks the old daemon workflow report labels.
- Runtime artifact build requests no longer use `workflow_name` in the selected active
  engine/CLI artifact-construction paths. The shared artifact request and `ash run` construction
  path use `entry_name` for checked application entries, and the Phase 201 gate blocks
  reintroducing the old carrier in those paths.
- Typechecker instance/control-link carriers no longer use `workflow_type` for target entry
  instances. `Type::Instance`, `Type::InstanceAddr`, and `Type::ControlLink` store `entry_type`,
  and the Phase 201 gate blocks reintroducing the old carrier field in `ash-typeck::types`.
- Runtime spawn/instance carriers no longer use the `workflow_type` token in active core,
  interpreter, engine, parser-lift, or CLI value-conversion paths. Spawn AST/value carriers and
  runtime instance addresses use `entry_type`, and the Phase 201 gate blocks the old token in the
  selected active runtime paths.
- Runtime callable/admission name carriers no longer use `workflow_name` in active engine and
  interpreter source paths. Callable-entry registration/lookup and application admission requests
  use `entry_name`, and the Phase 201 gate blocks the old token in those selected runtime paths.
- Runtime callable registry APIs no longer use stale callable-workflow identifiers in active
  engine/interpreter source paths. The registry type, storage, registration, blocking
  registration, and lookup APIs use callable-entry names, and the Phase 201 gate blocks the old
  callable-workflow registry identifiers.
- Runtime spawned-child registry APIs no longer use stale child-workflow identifiers in active
  engine/interpreter source paths. The registry storage, registration, lookup, engine embedding
  API, and spawned-child tests use child-entry names, and the Phase 201 gate blocks the old
  child-workflow registry identifiers.
- Runtime projection wrapper APIs no longer use stale workflow-projection names in active
  engine/interpreter wrapper paths. The interpreter wrapper module, exported functions, engine
  forwarding API, focused tests, and unsupported diagnostic label use entry-projection vocabulary,
  and the Phase 201 gate blocks the old workflow-projection wrapper names.
- TCIR/AMIR artifact carriers no longer use workflow-artifact names in active core/typechecker
  paths. Computation expressions, AMIR/bytecode statement/opcode variants, typechecker elaboration
  results, runtime artifact construction, focused tests, and TASK-931 evidence now use
  entry-artifact vocabulary, and the Phase 201 gate blocks the old artifact carrier tokens.
- Engine ordinary-source loading no longer names current entry/module source as workflow source.
  `LoadedOrdinaryFile` carries `ordinary_source`, the import-aware parser helper is
  `parse_entry_source_with_imports`, and the Phase 201 gate blocks the old source-loader names in
  active engine paths.
- Engine module loading no longer labels current ordinary source files, source snapshots, or
  parent-path diagnostics as workflow files/paths. The Phase 201 gate blocks the stale
  module-loader path/file labels in active engine code.
- Engine module export collection no longer carries the old `Act` opaque-type compatibility
  exception. Private ordinary aliases are not exportable/importable downstream by tower name, and
  module-loader fixtures now use target callable syntax and neutral builtin handle names.
- Additional active engine and CLI fixtures no longer carry removed callable syntax or workflow
  file/path labels in the selected paths covered by the Phase 201 gate. Inline callable,
  selected-evidence monomorphize, engine source, stdlib algebra signature, runtime-boundary, and
  JSON path tests use target callable syntax plus source/entry wording.
- Active standard-library README signature tables no longer use removed `Fun(...)` callable
  notation or bare unary arrow forms for `Option`/`Result` helpers. The Phase 201 gate blocks
  those stale table forms in `std/README.md`.
- Selected active parser and engine labels no longer describe current check targets, where-bound
  parsing, or ModuleFile parse authority as legacy behavior. The Phase 201 gate blocks those
  stale labels in the parser surface, H12 parser tests, and module-file check tests.
- Selected active TypeEnv fallback-boundary labels no longer describe current Type unifier and
  guarded normalizer rollout behavior as legacy. The Phase 201 gate blocks those stale labels in
  the TypeEnv tests and helper comments.
- Typechecker ambient effect context no longer uses workflow-effect carrier names. `TypeEnv`
  stores `ambient_effect`, callers use `set_ambient_effect`, runtime and obligation checks report
  `entry_effect`, and the Phase 201 gate blocks the old effect-carrier token in active
  typechecker/runtime paths.

## Follow-Up

TASK-1961 has a current green active-source gate, but Phase 201 remains open until TASK-1963
through TASK-1966 finish the internal carrier, vocabulary, tooling, and documentation quarantine
work and TASK-1968 runs the full closeout gates.
