# Product Requirements Document: Agent Semantic Workspace

**Status:** Initial working draft
**Maturity:** Exploratory / pre-standardization
**Primary pilot:** The Ash language project
**Initial language targets:** Rust and Ash
**Provisional language targets:** Python and Bash
**Initial project targets:** Rust projects, Ash projects, and mixed Rust/Ash workspaces

## 1. Executive summary

The Agent Semantic Workspace is a persistent, evidence-backed working environment that helps language-model agents understand, explore, modify, and verify software projects without repeatedly reconstructing project knowledge from files and text search.

The workspace is wider than retrieval-augmented generation and higher-level than a collection of Language Server Protocol operations. It combines:

- language-aware semantic analysis;
- project and documentation knowledge;
- current and target language semantics;
- project requirements, plans, tasks, and decisions;
- build, test, diagnostic, and runtime evidence;
- cross-language relationships;
- context selection under an explicit token budget;
- change planning and verification.

The workspace presents a small, provisional agent-oriented interface while allowing language semantics, provider adapters, bridge analyzers, recipes, and project services to evolve independently. It is explicitly not intended to become a standardized protocol before real usage demonstrates which abstractions work.

The first primary target is development of the Ash language project. That workspace requires two distinct modes:

1. **Developing Ash:** evolving the language definition, Rust implementation, tooling, documentation, and conformance corpus.
2. **Developing in Ash:** building Ash programs and Ash libraries using the currently supported language.

The same workspace must also support ordinary Rust projects. Python and Bash are included provisionally to ensure the architecture does not assume that all languages have strong static semantics or comprehensive language servers.

## 2. Problem statement

Coding agents currently interact with projects through combinations of file reads, text search, shell commands, LSP calls, documentation retrieval, and user-provided instructions. These mechanisms expose useful facts but leave the agent responsible for assembling a coherent working model of the project.

This creates recurring failures:

- The agent searches textually for relationships already known to a semantic analyzer.
- It uses file and line positions where a stable semantic entity is intended.
- It reads excessive context to understand a small change.
- It treats missing analyzer results as proof that no relationship exists.
- It confuses current behavior, target semantics, project intent, and tool support.
- It retrieves superseded specifications or historical plans as if they were current.
- It cannot tell whether a feature is specified, parsed, type-checked, lowered, executed, documented, or understood by tooling.
- It misunderstands a plan because goals, constraints, non-goals, dependencies, and verification obligations are embedded in prose.
- It misses relationships crossing Rust, Ash, Python, Bash, configuration, generated artifacts, and command invocations.
- It repeats previously gathered facts after context compaction or agent handoff.
- It declares completion without connecting requirements to implementation and verification evidence.

These problems are particularly visible in the Ash project because:

- Ash is not substantially represented in model training data.
- The language, compiler, tooling, reference, and project practices evolve together.
- The compiler and tooling are primarily written in Rust while programs, examples, and libraries are written in Ash.
- New Ash features may be defined before Ash semantic services understand them.
- Specifications, implementation, plans, tasks, examples, tests, and tooling can temporarily disagree.
- The repository contains normative, advisory, planned, historical, superseded, and exploratory material that must not be treated as equally authoritative.

## 3. Product vision

The Agent Semantic Workspace should function as the agent equivalent of the integrated working environment an IDE provides to a human, extended with capabilities IDEs usually lack:

- explicit project intent and requirement context;
- authority-aware specifications and reference material;
- cross-language and cross-domain relationships;
- truth-context and provenance explanations;
- persistent task state;
- goal-specific context compilation;
- transactional change planning;
- evidence-backed verification.

The workspace should answer not merely “what files match this query?” but:

> What does the agent need to know or do next, and which combination of retrieval, semantic computation, observation, change, and verification can establish it within the available context budget?

## 4. Goals

### 4.1 Generic workspace

Create a language- and tool-extensible workspace architecture that is not specific to Ash, Rust, LSP, or a single agent client.

### 4.2 Strong initial Rust and Ash support

Support:

- Rust projects through rust-analyzer, Cargo, rustc diagnostics, tests, and source analysis;
- Ash projects through native Ash analysis, the Ash reference corpus, Ash project metadata, and relevant LSP/MCP capabilities;
- mixed Rust/Ash workspaces, especially development of the Ash language itself.

### 4.3 Provisional Python and Bash support

Establish degraded but honest support for Python and Bash to test dynamic, optional-typing, scripting, command, and artifact-oriented semantics.

### 4.4 Low-token context learning

Help agents progressively learn the relevant subset of a language and project while minimizing repeated or irrelevant context.

### 4.5 Explicit contextual truths

Distinguish normative meaning, current implementation, target behavior, project requirements, tooling coverage, observation, history, and inference.

### 4.6 Evidence-driven evolution

Derive operations, semantic concepts, profiles, and recipes from real questions and agent traces rather than attempting to design a complete universal ontology upfront.

### 4.7 Safe change and verification

Eventually support impact analysis, change planning, preview, application, and verification while preserving workspace snapshots, provenance, and explicit limitations.

## 5. Non-goals

The initial product will not:

- define an industry standard or stable protocol;
- replace LSP or existing language tools;
- require all languages to share the same semantic ontology;
- guarantee full static understanding of dynamic languages;
- provide a universal project-management replacement;
- autonomously mutate external project-management or collaboration systems;
- prove behavioral equivalence of arbitrary program changes;
- build a complete knowledge graph before use cases justify it;
- expose every LSP method as a separate agent tool;
- make an LLM-generated explanation authoritative merely because it is plausible.

## 6. Product principles

### 6.1 Standardize form, specialize meaning

The workspace may standardize entity handles, evidence, snapshots, budgets, completeness, changes, and verification results. Language profiles define what concepts and relationships mean.

### 6.2 Treat LSP as a provider, not the product model

LSP is a valuable source of semantic facts. It remains editor-oriented and position-centric. The workbench should place task- and entity-oriented operations above it.

### 6.3 Preserve language-native concepts

Rust traits, blanket implementations, macro expansions, and trait obligations should remain Rust concepts. Ash tasks, actors, capabilities, effects, obligations, policies, processes, channels, and lowering relationships should remain Ash concepts.

### 6.4 Make uncertainty explicit

“No result,” “unsupported,” “ambiguous,” “provider failure,” and “invalid program” are different outcomes.

### 6.5 Keep providers honest

Every provider must advertise feature- and configuration-sensitive coverage. An analyzer must not imply knowledge of a language feature it does not model.

### 6.6 Make context a compiled product

The workspace should select, summarize, group, and cite the smallest useful set of facts and evidence for the agent’s next decision.

### 6.7 Preserve raw evidence

Synthesized results should retain references to the provider facts, source excerpts, commands, revisions, and observations from which they were derived.

### 6.8 Prefer reversible architecture

Tool names, semantic taxonomies, recipes, pack formats, graph structures, and planning strategies remain experimental until validated across languages and tasks.

## 7. Users and primary activities

### 7.1 Language implementer

Develops the Ash language definition, compiler, runtime, or tooling. Needs to trace a construct across specification, Rust implementation, Ash examples, diagnostics, tests, reference documentation, and project tasks.

### 7.2 Ash programmer

Develops programs or ordinary libraries in Ash. Needs current user-facing language semantics, library APIs, diagnostics, semantic navigation, refactoring, and verification without compiler-internal details.

### 7.3 Ash standard-library developer

Develops in Ash while also maintaining platform-facing APIs, conformance behavior, compatibility, and possible compiler/library boundaries.

### 7.4 Rust programmer

Develops a Rust project using rust-analyzer and Cargo-derived semantics. Needs symbol-oriented navigation, trait and macro understanding, impact analysis, and configuration-aware verification.

### 7.5 Polyglot project contributor

Works across Rust, Ash, Python, Bash, manifests, CI, generated artifacts, and subprocess boundaries.

### 7.6 Test or review agent

Needs a different projection of the same task: invariants, failure modes, affected surfaces, evidence, and completion criteria rather than implementation-only context.

## 8. Distinguishing developing Ash from developing in Ash

The workspace must model three connected layers:

1. **Language contract:** specifications and reference defining Ash.
2. **Host implementation:** Rust compiler, runtime, CLI, LSP, MCP, and analysis services implementing Ash.
3. **Target ecosystem:** Ash programs, examples, tests, workflows, and libraries.

### 8.1 Developing Ash

Relevant questions include:

- Where is a language construct specified?
- Is it current, target, deprecated, or historical?
- Where is it parsed, resolved, checked, lowered, and executed?
- Which diagnostics and examples define its boundaries?
- Which Rust symbols implement it?
- Which Ash examples and tests establish conformance?
- Which references and migration material must change?
- Which semantic services understand it today?

The context policy should include normative rules, current/target deltas, Rust implementation, conformance Ash code, project plans, and verification gates.

### 8.2 Developing in Ash

Relevant questions include:

- How is a behavior expressed using supported Ash?
- What are the type, effect, capability, obligation, or workflow implications?
- Where is an Ash entity defined and used?
- Why does a program fail to check or run?
- Which tests exercise a library or workflow?
- What breaks if an API changes?

The context policy should normally hide historical designs, superseded specifications, compiler internals, and active language plans unless a likely language or tooling defect is encountered.

### 8.3 Ash platform libraries

Standard or language-defining Ash libraries occupy both domains. They are ordinary Ash code and public platform artifacts. Their context may include public API compatibility, specification obligations, compiler special treatment, bootstrap constraints, and conformance evidence.

## 9. Contextual truth model

The workspace must store contextual claims rather than one undifferentiated truth.

### 9.1 Truth realms

- language definition;
- language implementation;
- programs and libraries in the target language;
- semantic tooling model;
- project intent and requirements;
- project status;
- user guidance;
- tests, builds, and runtime observations;
- historical decisions;
- workbench or agent inference.

### 9.2 Claim modalities

- **Normative:** required by a current specification.
- **Required:** demanded by an accepted project requirement or task.
- **Planned:** intended future work.
- **Implemented:** present in current code.
- **Observed:** demonstrated under a particular build, test, or execution.
- **Tool-supported:** understood by a named provider.
- **Documented:** stated in current user guidance.
- **Deprecated:** supported temporarily but scheduled for removal.
- **Superseded:** no longer authoritative within a declared scope.
- **Historical:** previously applicable or explanatory.
- **Inferred:** concluded from evidence but not authoritative.
- **Hypothetical:** evaluated under a proposed change.

### 9.3 Contextual worlds

- released;
- current workspace or branch;
- target semantics;
- historical revision;
- hypothetical change.

### 9.4 Query-dependent authority

Authority depends on the question:

| Question | Primary authority |
| --- | --- |
| What should this Ash construct mean? | Current normative specification |
| What does the current branch do? | Implementation plus test/runtime evidence |
| What work is required? | Accepted task, requirement, and target specification |
| What can the semantic tooling analyze? | Provider capability and evaluation evidence |
| What should users write today? | Supported implementation plus current reference |
| Why was this design chosen? | Design and decision history |
| Is a feature complete? | Declared completion contract plus evidence |

### 9.5 Conflict handling

The workspace must represent disagreement directly. It must not silently collapse a target specification, partial implementation, stale reference, and syntax-only LSP result into one answer.

## 10. Feature-development model

Language features may advance independently across several dimensions:

- requirement accepted;
- semantics specified;
- syntax/parser implemented;
- identity/name resolution implemented;
- static semantics implemented;
- lowering implemented;
- runtime implemented;
- diagnostics implemented;
- LSP support implemented;
- agent-workbench support implemented;
- reference and migration material current;
- positive, compositional, negative, and runtime tests present.

The workspace must expose a feature-level coverage matrix rather than one status field.

Semantic queries should support at least these modes:

- **current:** answer according to current implementation;
- **target:** answer according to accepted target semantics;
- **delta:** explain differences and remaining work;
- **auto:** select a mode based on task and component role, then state the selection.

Providers must be able to return `unsupported_feature` without treating the source as invalid or reporting an empty semantic result.

Provisional semantic overlays may teach and syntactically navigate a target feature before native semantic support exists. Overlay-derived claims must remain advisory and visibly distinct from compiler-derived facts.

## 11. Conceptual architecture

The initial architecture consists of the following logical layers.

### 11.1 Workspace runtime

Owns:

- workspace and component identity;
- composite snapshots and freshness;
- provider lifecycle;
- normalized evidence and claim records;
- task context ledger;
- query planning;
- context compilation;
- change-set and verification state;
- permissions and action boundaries.

### 11.2 Semantic profiles

Describe language-native entities, relationships, views, soundness rules, common questions, and interpretation guidance independently of a specific tool.

### 11.3 Provider adapters

Obtain facts from:

- standard LSP servers;
- language-server-specific extensions;
- native compiler or analysis services;
- CLI/build/test tools;
- source and structural search;
- documentation and project-management services;
- runtime observation.

### 11.4 Recipes

Describe how provider facts can answer compound questions. Recipes may initially be code, structured configuration, or both. No recipe language is standardized in the MVP.

### 11.5 Cross-language bridge analyzers

Connect semantic islands through:

- process and CLI invocation;
- environment variables and configuration keys;
- generated files and artifact flows;
- serialization and schema correspondence;
- FFI and bindings;
- network endpoints;
- database tables and migrations;
- CI and build orchestration.

### 11.6 Corpus and project services

Provide authority-aware specifications, references, plans, tasks, decisions, deprecations, supersession, drift, completion criteria, and historical context.

### 11.7 Agent-facing facade

Presents a small experimental set of entity-, task-, question-, and context-oriented operations through MCP or another client transport.

## 12. Profiles, adapters, and service ownership

### 12.1 Workbench-owned artifacts

The generic workbench should initially own:

- experimental query/result contracts;
- claim, evidence, snapshot, and completeness models;
- generic LSP adapter;
- generic filesystem/search adapters;
- pack and provider discovery;
- common bridge analyzers;
- trace and evaluation harness.

### 12.2 Language-owned artifacts

A language project should ideally own:

- its canonical semantic profile;
- native provider integration;
- language-specific recipes;
- reference-resource mapping;
- feature coverage declarations;
- conformance and evaluation cases.

Ash should own the canonical Ash profile and native Ash provider.

### 12.3 Tool- or community-owned artifacts

External packs may provide rust-analyzer, Pyright, Bash language server, ShellCheck, framework, or specialized bridge integrations.

### 12.4 Workspace-owned configuration

The project repository selects providers, configurations, experimental features, component roles, bridge declarations, and verification gates. It should not duplicate entire language profiles.

## 13. Multi-language workspace requirements

### 13.1 Component classification

Every component should declare or infer:

- implementation language;
- project role;
- package, executable, library, script, workflow, test, generated, documentation, or tooling identity;
- relevant configuration and build targets.

Example roles include:

- `language_definition`;
- `language_implementation`;
- `semantic_tooling`;
- `standard_library`;
- `target_library`;
- `application`;
- `conformance_example`;
- `project_automation`;
- `project_tooling`;
- `user_reference`.

### 13.2 Global and provider-local identities

The workspace needs global handles that can retain provider-local identity and source locations. Stable identity format remains experimental.

### 13.3 Composite snapshots

A snapshot may contain independent revisions for Git, dirty files, semantic databases, language servers, issue trackers, documentation indexes, builds, and CI results. Claims must carry the revisions relevant to them.

### 13.4 Cross-language impact

Impact analysis must be able to connect a Rust CLI declaration to Bash invocations, Python wrappers, Ash tasks, CI configuration, and tests.

## 14. Language-target requirements

### 14.1 Rust

Initial Rust support should combine:

- rust-analyzer standard LSP capabilities;
- selected rust-analyzer extensions when justified and available;
- Cargo metadata and configuration;
- `cargo check`, tests, and Clippy;
- source evidence and structural search.

Initial Rust semantic concerns include:

- modules, functions, types, traits, implementations, associated items, and macros;
- definitions, references, implementations, call hierarchy, and related tests;
- generic bounds and failed trait obligations;
- macro expansion;
- feature, target, build-script, and proc-macro configuration;
- public API and change impact.

### 14.2 Ash

Initial Ash support should combine:

- native parser, resolver, type/effect checker, lowering, runtime, and policy services as available;
- `ash-lsp-core` and `ash-mcp` capabilities;
- feature-coverage declarations;
- authority-aware specification and reference corpus;
- project plans, tasks, decisions, supersession, and drift;
- Rust/Ash traceability bridges;
- examples and conformance corpus.

Initial Ash semantic concerns include:

- functions, types, kinds, values, macros, tasks, actors, roles, permissions, capabilities, effects, obligations, policies, processes, channels, workflows, provenance, and lowering;
- syntax/static/dynamic semantics distinctions;
- current/target/delta analysis;
- implementation and tooling coverage per feature.

### 14.3 Python, provisional

Python support should test optional typing and dynamic behavior through a combination such as Pyright, Python AST/import analysis, source search, and pytest. Results must identify blind spots including dynamic imports, monkey-patching, reflection, and dynamic attribute access.

### 14.4 Bash, provisional

Bash support should emphasize scripts, functions, sourced files, variables, command invocation, environment, pipelines, artifacts, and ShellCheck diagnostics. It should not pretend to offer Rust-like symbol or call-graph certainty.

## 15. Context compiler requirements

The workspace should maintain substantial semantic state outside the model’s immediate context and project only what is useful for the current goal.

### 15.1 Context inputs

- task goal;
- starting entities, files, concepts, requirements, or diagnostics;
- activity mode;
- target audience or agent role;
- workspace snapshot;
- prior task ledger;
- token and result budgets;
- desired evidence strength.

### 15.2 Context outputs

- interpretation header;
- concise task or topic summary;
- relevant entity cards;
- important relationships;
- current and target semantics where relevant;
- project requirements and constraints;
- exact evidence excerpts;
- known divergences and unsupported features;
- omitted-result groups with expansion handles;
- suggested next questions or operations;
- verification obligations;
- tokens used and snapshot identity.

### 15.3 Progressive disclosure

Results should support:

- orientation summary;
- working detail;
- exact evidence;
- raw provider detail on demand.

### 15.4 Task ledger

The workspace should persist accepted facts, user decisions, agent hypotheses, affected entities, unresolved questions, change plans, and verification results. Hypotheses must not silently become facts.

### 15.5 Delta context

After edits, context should emphasize semantic and diagnostic changes since the prior snapshot rather than retransmitting the complete project model.

## 16. Experimental agent interface

The exact interface remains an experiment. The initial system should make competing shapes inexpensive to test.

Candidate stable concerns include:

- workspace orientation and health;
- entity resolution;
- entity inspection;
- relationship exploration;
- context construction;
- explanation;
- change analysis and planning;
- verification.

Candidate provisional operations include:

- `workspace`;
- `resolve`;
- `inspect`;
- `related`;
- `context`;
- `explain`;
- `change`;
- `verify`.

The system may also expose primitive, compound, graph-oriented, and goal-oriented experimental variants simultaneously for evaluation.

Every substantial result should include:

- summary;
- structured data;
- evidence;
- snapshot;
- confidence;
- completeness;
- warnings and limitations;
- omitted groups or continuation handles;
- suggested next operations.

## 17. Project and knowledge services

The workspace should initially provide read-oriented project services:

- resolve a requirement, issue, plan, task, decision, or specification;
- retrieve goals, constraints, dependencies, non-goals, acceptance criteria, and verification gates;
- determine authority, status, health, and supersession;
- connect project artifacts to code, tests, changes, and completion evidence;
- compile task-oriented briefs;
- detect status/evidence mismatches.

External project mutations, comments, assignments, notifications, and status changes are deferred and must require explicit authorization when introduced.

## 18. Functional requirements

### FR-1: Workspace discovery

The system shall discover or accept explicit workspace components, languages, roles, providers, build configurations, and project services.

### FR-2: Provider capability negotiation

The system shall record provider capabilities, versions, feature coverage, configuration, health, and stability.

### FR-3: Entity resolution

The system shall resolve names, qualified names, locations, concepts, and project artifacts into candidate entities with reasons and ambiguity reporting.

### FR-4: Semantic inspection

The system shall return language-native views of supported entities with exact evidence and completeness information.

### FR-5: Relationship exploration

The system shall explore bounded language-local and cross-language relationships with ranking, grouping, and continuation.

### FR-6: Context compilation

The system shall construct task-specific context within an explicit budget and support progressive expansion.

### FR-7: Truth-context explanation

The system shall state the realm, world, modality, authority, revision, and evidence basis of material claims when these affect interpretation.

### FR-8: Feature coverage

The system shall distinguish unsupported, partial, ambiguous, invalid, and failed analysis and shall expose feature-level coverage for evolving languages.

### FR-9: Task preparation

The system shall compile requirements, plans, current state, target state, implementation surfaces, constraints, and verification obligations into a bounded task brief.

### FR-10: Persistent task state

The system shall preserve task findings and decisions outside the immediate model context and invalidate or refresh them when relevant sources change.

### FR-11: Change analysis

After the read-only substrate is proven, the system shall support impact analysis and immutable change plans tied to a base snapshot.

### FR-12: Verification

The system shall select and execute proportionate checks, compare against a baseline, and report verified and unverified configurations separately.

### FR-13: Traceability

The system shall retain provenance from synthesized answers to provider facts and source evidence.

### FR-14: Evaluation traces

The system shall capture sufficient trace data to compare alternative interface and recipe designs without exposing sensitive content unnecessarily.

## 19. Non-functional requirements

### 19.1 Token efficiency

The system should minimize irrelevant and repeated context while retaining sufficient evidence for reliable decisions.

### 19.2 Latency

Interactive navigation and context-expansion operations should exploit persistent provider processes, caches, and incremental analysis.

### 19.3 Correctness and honesty

Incomplete analysis must be reported as incomplete. Empty results must not conceal unsupported semantics or provider failure.

### 19.4 Isolation

External providers should generally run in persistent, cancellable, timeout-bounded processes. The design should avoid an unstable in-process plugin ABI as an initial requirement.

### 19.5 Security

The system must enforce workspace-root containment, explicit command templates, trust boundaries for build scripts and plugins, and separate read, proposed change, local mutation, and external mutation capabilities.

### 19.6 Reproducibility

Material conclusions and verification results should identify relevant source revisions, configurations, provider versions, and command invocations.

### 19.7 Extensibility

Adding a language or service should not require changes to every client-facing operation. Unsupported concepts may remain profile-specific extensions.

## 20. MVP proposal

The MVP should be an Ash-first, read-oriented vertical slice with Rust support strong enough to develop the Ash implementation.

### 20.1 MVP capabilities

1. Persistent workspace and provider lifecycle.
2. Composite workspace snapshot.
3. Component and role classification.
4. Generic LSP adapter.
5. rust-analyzer and Cargo provider integration.
6. Ash native/LSP semantic provider integration using currently available capabilities.
7. Authority-aware Ash corpus and project-artifact index.
8. Current/target/delta feature status.
9. Entity resolution and compact entity cards.
10. Definitions, grouped references, related definitions, and relevant tests where supported.
11. Task-context compilation under a token budget.
12. Persistent task ledger.
13. Provider and claim provenance.
14. Trace-based evaluation harness.

### 20.2 MVP compound use cases

#### UC-1: Prepare an Ash language-development task

Given an Ash task identifier, return the goal, target semantic delta, current implementation state, relevant Rust entities, Ash examples/tests, known drift, tooling gaps, and completion contract.

#### UC-2: Trace an Ash language concept

Given an Ash concept, connect normative specification, current/reference status, Rust parser/resolver/checker/lowering/runtime implementation, Ash examples, diagnostics, and tests.

#### UC-3: Understand a Rust entity

Given a Rust name or location, resolve its identity and return definition, signature, implementations, grouped references, callers/callees where supported, related tests, configuration basis, and limitations.

#### UC-4: Understand an Ash entity

Given an Ash name or location, return definition, identity, related entities, supported semantic views, references, relevant language card, specification links, and unsupported-feature warnings.

#### UC-5: Resume a task cheaply

Given a task-ledger handle, return a compact current summary, accepted facts, decisions, open questions, affected entities, changes since the previous snapshot, and next verification step.

### 20.3 Explicit MVP exclusions

- automatic code mutation;
- external project-management mutation;
- complete Python or Bash semantic navigation;
- general runtime tracing;
- stable public pack or recipe specification;
- guaranteed stable cross-edit semantic identifiers;
- standardized protocol commitment.

## 21. Delivery phases

### Phase 0: Question and trace corpus

- Collect real Ash and Rust development traces.
- Infer latent questions rather than copying existing tool calls.
- Record expected answers, evidence, failure modes, context cost, and follow-up sequences.
- Establish baseline workflows using files, search, LSP, and current MCP tools.

### Phase 1: Workspace and evidence substrate

- Implement provider lifecycle, snapshots, component roles, normalized evidence, capability declarations, and trace capture.
- Support the generic LSP and Cargo providers.

### Phase 2: Ash project context

- Index authority, status, health, supersession, implementation traceability, plans, tasks, and feature coverage.
- Implement task preparation and concept tracing.

### Phase 3: Context compiler

- Implement token-budgeted context bundles, entity cards, grouping, expansion handles, task ledger, and delta refresh.

### Phase 4: Semantic exploration

- Improve Rust and Ash resolution, references, implementations, tests, and cross-language Rust/Ash bridges.
- Add provisional Python/Bash providers and command/artifact bridges.

### Phase 5: Change and verification experiments

- Add impact analysis, plan, preview, controlled application, and proportionate verification after read-oriented benchmarks meet quality thresholds.

## 22. Evaluation strategy

The product should be evaluated on real questions, not protocol elegance.

### 22.1 Core metrics

- task-answer correctness;
- required-evidence recall;
- false claims of absence or completeness;
- tool calls per task;
- context tokens consumed;
- repeated-fact transmission;
- irrelevant-context ratio;
- latency;
- stale-context incidents;
- cross-language relationships missed;
- incorrect source-authority selection;
- agent recovery after compaction or handoff;
- verification coverage and unverified-scope honesty.

### 22.2 Competing interaction experiments

Evaluate at least:

- raw files and search;
- thin LSP primitives;
- fixed compound operations;
- semantic graph queries;
- question/recipe-based operations;
- goal-oriented context requests.

Different approaches may win for different tasks. The architecture must permit a hybrid result.

### 22.3 Promotion criteria

A contract element should be considered for stability only after it:

- appears across multiple languages;
- supports multiple question families;
- has at least two provider implementations or meaningful fallbacks;
- has understood failure and completeness semantics;
- performs well in trace-based evaluations;
- has survived competing representations.

## 23. Success criteria for the initial Ash pilot

The initial pilot is successful when:

1. An agent can begin a representative Ash language-development task without reading broad unrelated repository sections.
2. The task context clearly distinguishes requirement, target semantics, current implementation, tooling support, reference state, and observed evidence.
3. The agent can trace at least one language concept from specification through Rust implementation to Ash conformance examples and tests.
4. The agent can resolve and inspect relevant Rust and Ash entities without first grepping for exact locations.
5. Unsupported new Ash features produce explicit degraded-analysis results rather than false absence or invalidity claims.
6. Median task-context use is materially lower than the current file/search baseline without reducing correctness.
7. Context can be resumed after compaction with no silent promotion of hypotheses to facts.
8. The system identifies at least one real status, specification, implementation, reference, or tooling-coverage mismatch in a development task.

## 24. Risks and mitigations

### 24.1 Premature universal ontology

**Risk:** Rust or Ash concepts are flattened into misleading generic categories.
**Mitigation:** Keep language profiles open-ended and standardize only common result mechanics.

### 24.2 Premature protocol standardization

**Risk:** Early API choices become compatibility burdens.
**Mitigation:** Mark contracts experimental, version aggressively, and compare multiple interaction models.

### 24.3 Tooling mistaken for semantic authority

**Risk:** LSP limitations are presented as language facts.
**Mitigation:** Require feature-level capability and completeness reporting.

### 24.4 Stale or superseded corpus retrieval

**Risk:** An agent follows historical Ash semantics.
**Mitigation:** Make authority, status, health, scope, and supersession explicit retrieval constraints.

### 24.5 Excessive system scope

**Risk:** The project becomes a universal knowledge and project-management platform before proving code-context value.
**Mitigation:** Start with read-oriented Ash/Rust task context and semantic navigation; defer external mutation and broad integrations.

### 24.6 Expensive context synthesis

**Risk:** The workbench uses more latency or tokens than direct tools.
**Mitigation:** Cache entity cards and provider facts, use delta results, rank by utility per token, and benchmark against baselines.

### 24.7 Incorrect synthesized explanation

**Risk:** A concise explanation hides erroneous reasoning.
**Mitigation:** Reserve context budget for exact evidence and preserve raw provider fact handles.

### 24.8 Unstable provider internals

**Risk:** rust-analyzer or compiler-internal integrations create maintenance burden.
**Mitigation:** Begin with standard LSP and Cargo; place experimental extensions behind adapter capability boundaries.

### 24.9 Unsafe execution

**Risk:** providers, build scripts, or commands execute untrusted workspace behavior.
**Mitigation:** explicit trust, command policies, isolation, timeouts, and permission separation.

## 25. Open questions

1. Should the first implementation live inside the Ash repository, as a separate experimental repository, or as a workbench core with an Ash pack co-located in Ash?
2. What is the minimum normalized entity model needed before language-native extensions become unwieldy?
3. How stable can semantic handles be across edits without requiring compiler-internal persistent identities?
4. Should initial recipes be implemented in Rust, structured data, Ash workflows, or a mixture?
5. Which project metadata should be hand-authored versus generated from existing Ash documents?
6. How should provisional target-semantics overlays be validated and retired?
7. What token-budget allocation best balances summary, code evidence, reference guidance, and limitations?
8. Which rust-analyzer extensions are valuable enough to support despite experimental contracts?
9. How should the workspace represent multiple worktrees with different effective versions of Ash?
10. When should Python and Bash support move beyond bridge and diagnostic use cases?
11. Which task-context operations should be dedicated tools versus views of a generic context operation?
12. What evidence threshold is required before the workbench may recommend that a task is complete?

## 26. Recommended immediate next steps

1. Select three to five completed Ash development tasks representing syntax, static semantics, runtime behavior, tooling, and documentation evolution.
2. Reconstruct the latent questions and failure points from their traces or repository history.
3. Define a minimal feature-development packet for one recent Ash feature.
4. Prototype a read-only `prepare_task` result from existing Ash specs, plans, tasks, code, and tests.
5. Prototype a `trace_concept` result connecting one Ash concept to Rust and Ash artifacts.
6. Establish token, correctness, evidence, and authority-selection baselines.
7. Use those results to decide the first internal query contracts rather than finalizing the protocol in advance.

## 27. Working product hypothesis

The current hypothesis is:

> A persistent semantic workspace that compiles goal-specific, authority-aware, evidence-backed context from language services, project knowledge, and verification systems will allow agents to work more accurately with substantially less repeated context than file/search- or LSP-only workflows.

This hypothesis remains to be tested. The first durable product may be the workspace, evidence, trace, and evaluation substrate rather than any particular agent-facing tool taxonomy.
