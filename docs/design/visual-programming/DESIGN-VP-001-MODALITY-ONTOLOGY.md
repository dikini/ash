# DESIGN-VP-001: Modality Ontology for Ash Interfaces

## Status: Draft

## 1. Problem Statement

Different interface modalities — text, node-graph, object inspector, SCADA panel, dashboard — excel at different semantic scopes, yet each fails at others. Before designing Ash's visual programming surfaces, we need a shared ontology for comparing them.

This document defines:
- a set of comparative dimensions,
- a mapping of five canonical modalities against those dimensions,
- what Ash should steal from each,
- and how the modalities layer together in an integrated Ash environment.

## 2. The five canonical modalities

| Modality | Exemplars | Core abstraction | Native time scope |
|----------|-----------|------------------|-------------------|
| **Text** | Lisp, Rust, Ash `.ash` | Abstract syntax tree / symbol graph | Design-time, versioned |
| **Node-graph** | Node-RED, n8n, Houdini, Unreal Blueprints | Dataflow topology / message pipeline | Design-time + runtime overlay |
| **Object-space** | Smalltalk-80, Self, Pharo, Factor listeners | Live object heap / inspector browser | Runtime-primary (design is mutation) |
| **SCADA / HMI** | Wonderware, Ignition, ladder logic, SFC | Process state / supervisory control loop | Runtime-primary, historical trending |
| **Dashboard** | Grafana, Datadog, Excel | Aggregate metrics / time-series snapshots | Runtime-monitoring, post-hoc analysis |

## 3. Classification dimensions

### Axis A: Liveness (when does representation match reality?)
- **Dead** (static): Text files, committed graph JSON.
- **Live** (runtime-reflective): Smalltalk inspectors, SCADA tags, REPL sessions.
- **Delayed** (trailing): Dashboards, trace viewers, audit logs.

### Axis B: Scope granularity (smallest visible/editable unit)
- **Atom** (expression-level): Text cursor, Smalltalk inspector field.
- **Form** (statement/workflow-level): Node-graph node, ladder rung.
- **Module** (file/package-level): Smalltalk browser category, SCADA screen.
- **System** (distributed topology): Node-RED full-deploy view, SCADA plant overview.

### Axis C: Semantic fidelity (directness of mapping to execution model)
- **Isomorphic** (1:1 with runtime): Smalltalk object graph, SCADA tag database.
- **Homomorphic** (structured projection): Node-graph dataflow ≈ async message topology.
- **Symbolic** (conventional representation): Text syntax, dashboard panel.
- **Metaphoric** (analogical): Game engines, SCADA P&ID diagrams.

### Axis D: Mutability gradient (edit → effect latency)
- **Edit→Restart**: Text (compile, deploy, run).
- **Edit→Hot-swap**: Node-graph (deploy patch while running).
- **Edit→Instant**: Smalltalk (modify method, all objects use new behavior immediately).
- **Read-only**: Dashboard (observe, configure view, but not program logic).

### Axis E: Composability (how larger things are built from smaller ones)
- **Lexical nesting**: Text blocks, functions, modules.
- **Spatial composition**: Node-graph wires, SCADA screen panels.
- **Reference linking**: Smalltalk message sends, object pointers.
- **Temporal composition**: Dashboard time ranges, SCADA recipe sequences.

### Axis F: Governance suitability (authority, policy, audit visibility)
- **Excellent**: SCADA (alarms, permissions, operator actions logged by design).
- **Good**: Text (diffs show who changed what, but policies are implicit in code).
- **Poor**: Node-graph (who approved this wire? when? usually invisible).
- **Mixed**: Dashboard (good for post-hoc audit, bad for preventive governance).

## 4. Per-modality analysis

### 4.1 Text (Ash source)
- **Targets**: Precision, abstraction, version control, diff-based review, batch reasoning.
- **Excels at**: Generic interfaces, type schemes, policy logic, proof targets.
- **Fails at**: Runtime topology, temporal dynamics, system-level gestalt, live debugging.
- **Hidden strength**: It is the only modality where **obligation and policy can be parameterized** naturally (e.g. `require_approval(role: T)`).

### 4.2 Node-graph
- **Targets**: Dataflow intuition, rapid pipeline assembly, visual debugging of message paths.
- **Excels at**: "What connects to what," parallel branches, I/O multiplexing.
- **Fails at**: Distinguishing definition/declaration/instance, nested scoping, type-parameterized generics, policy provenance.
- **Hidden strength**: It makes **effect escalation** viscerally obvious (red nodes downstream of blue nodes).

### 4.3 Object-space (Smalltalk-style)
- **Targets**: Live exploration, emergent understanding, removing the compile-run gap.
- **Excels at**: Inspection of any object at any time, debugging as a conversation with the system, "turtles all the way down."
- **Fails at**: Distributed systems, static analysis, git-style collaboration, security boundaries (the image is a single trust domain).
- **Hidden strength**: The **browser/inspector split** — code on one side, live objects on the other — is a paradigm Ash can borrow for the definition/instance split.

### 4.4 SCADA / HMI
- **Targets**: Safety-critical supervision, alarm management, operator situational awareness.
- **Excels at**: Real-time state display, historical trending, role-based operator actions, structured logging of every interaction.
- **Fails at**: General computation, abstraction, composable reuse, version control.
- **Hidden strength**: SCADA **tag historization** is essentially provenance-by-default — every sensor value is timestamped and logged. This maps directly to Ash's provenance requirements.

### 4.5 Dashboard
- **Targets**: Aggregate health, anomaly detection, executive summary.
- **Excels at**: Time-series comparison, correlation across metrics, "at a glance" status.
- **Fails at**: Causality, local reasoning, editing, composition.
- **Hidden strength**: It is the natural home for **policy dashboards** — "what percentage of decisions required approval this week?"

## 5. Ontological synthesis: three-layer lattice for Ash

Rather than choosing one modality, Ash should support a **multi-modal integrated environment** where each view is primary for a specific concern.

```
┌──────────────────────────────────────────────────┐
│  LAYER 3: SYSTEM                        │
│  Modality: SCADA/Dashboard hybrid       │
│  Concern: Running instances, supervision│
│  trees, alarms, policy violations       │
│  Time: Runtime + historical             │
├──────────────────────────────────────────────────┤
│  LAYER 2: MODULE                        │
│  Modality: Node-graph + text sidecar    │
│  Concern: Workflow definitions, static  │
│  topology, capability wiring, policy    │
│  composition                            │
│  Time: Design-time (with simulation)    │
├──────────────────────────────────────────────────┤
│  LAYER 1: OBJECT                        │
│  Modality: Smalltalk-style inspector    │
│  Concern: Individual bindings, trace    │
│  events, mailbox contents, capability   │
│  state                                  │
│  Time: Live runtime                     │
└──────────────────────────────────────────────────┘
```

**The Smalltalk principle applied:** The module graph is the live system. Editing a workflow definition is sending a message to the module system. The text editor and node-graph are lenses on the same live module object.

## 6. What Ash should steal from each modality

| Source | Principle | Ash application |
|--------|-----------|-----------------|
| **Smalltalk** | Inspector/browser duality | Split pane: definition on left, live instance inspector on right |
| **Smalltalk** | Everything is inspectable | Any value in a trace can be opened to see its provenance chain |
| **Smalltalk** | Method changes apply live | Hot-reload a workflow; instances optionally migrate or stay on old version |
| **SCADA** | Tag historization | Every capability call, policy decision, and role assignment is a historized "tag" |
| **SCADA** | Alarm priority + acknowledgment | Policy violations surface as alarms with required role acknowledgment |
| **SCADA** | Recipe / batch control | "Run this workflow with these parameters" as a batch recipe that can be scheduled |
| **SCADA mini-lang** | Ladder logic / SFC | A *secondary* visual syntax for simple sequential workflows (not primary) |
| **Node-graph** | Edge = dataflow | Inter-workflow message passing drawn as wires between instance nodes |
| **Node-graph** | Color = semantic category | Node fill = effect level; border = role authority; edge style = communication pattern |
| **Text** | Diffability | Graph edits must round-trip to text so `git diff` remains meaningful |

## 7. Open research questions

These questions must be resolved before any visual editor is implemented:

1. **Migration semantics.** If a workflow definition is edited live, what happens to running instances? Do they migrate, fork, or continue with the old code? (The Ash equivalent of Smalltalk `become:`.)

2. **Provenance in visual editing.** If a user drags a wire to bypass a policy check, how is the change attributed, timestamped, and justified? Node-graph version control is largely unsolved.

3. **Generic visualization.** How do you draw a generic `impl<T> Validator<T> where T: Serializable` in a node-graph? Wireframe ports that solidify on instantiation is one hypothesis.

4. **Time-travel debugging.** Can Ash reconstruct a workflow execution state from its Merkle provenance trace? If so, the dashboard becomes a time machine.

## 8. Relationship to other design notes

- [DESIGN-VP-002-REPL-NOTEBOOK.md](DESIGN-VP-002-REPL-NOTEBOOK.md) — explores how Layer 2 and Layer 1 are co-accessible through a notebook kernel.
- [DESIGN-VP-003-VISUAL-GRAMMAR.md](DESIGN-VP-003-VISUAL-GRAMMAR.md) — will concretize the node-graph vocabulary introduced here.
- [DESIGN-VP-004-ROUND-TRIP-FIDELITY.md](DESIGN-VP-004-ROUND-TRIP-FIDELITY.md) — will formalize the text-ground-truth requirement.
- [DESIGN-VP-005-LIVE-STATE-PROJECTION.md](DESIGN-VP-005-LIVE-STATE-PROJECTION.md) — will tackle the nondeterministic live-state serialisation problem.

## 9. References

- Kay, A. (1977). *Personal Dynamic Media*.
- Ingalls, D. et al. (2020). *Evolution of Smalltalk*.
- SCADA International / ISA-101 — HMI design standards.
- Node-RED documentation: https://nodered.org/docs/
- Glamorous Toolkit: https://gtoolkit.com/
- Ignition Perspective Module: https://inductiveautomation.com/ignition/
