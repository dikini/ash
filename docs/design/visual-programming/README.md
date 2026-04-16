# Visual Programming & Multi-Modal Interfaces for Ash

This directory collects research, design notes, and specifications for the visual and interactive surfaces of Ash.

## Core Principles

1. **Ground truth is Ash source text.** Every modality — node-graph, notebook cell, dashboard panel, inspector field — derives from, and must round-trip to, canonical `.ash` source.
2. **Visual programming and the REPL are co-designed.** The notebook interface (cell-based, incremental, eval/print-loop) and the visual editor (graph-based, spatial, topology-first) share a common backend: the Ash REPL kernel.
3. **Live state representation is a derived concern.** Serialising a live Ash environment (workflow instances, mailboxes, control links) is explicitly a projection problem. Some projections are deterministic (instance tree → JSON), others are intentionally nondeterministic (real-time dashboard sampling).
4. **No single modality wins.** Each surface excels at a specific semantic scope. The design goal is interoperability, not replacement.

## Document Index

| Document | Purpose | Status |
|----------|---------|--------|
| [`DESIGN-VP-001-MODALITY-ONTOLOGY.md`](DESIGN-VP-001-MODALITY-ONTOLOGY.md) | Comparative ontology of text, node-graph, object-space, SCADA, and dashboard modalities. | Draft |
| `DESIGN-VP-002-REPL-NOTEBOOK.md` | Notebook cell semantics, incremental parse/check/execute, and the kernel API. | TBD |
| `DESIGN-VP-003-VISUAL-GRAMMAR.md` | Concrete visual vocabulary for Ash: shapes, colors, edges, containment, and port typing. | TBD |
| `DESIGN-VP-004-ROUND-TRIP-FIDELITY.md` | Rules for text ↔ AST ↔ graph serialization, including span preservation and formatter contracts. | TBD |
| `DESIGN-VP-005-LIVE-STATE-PROJECTION.md` | How to represent and serialise running workflow instances, control links, and traces. | TBD |
| `DESIGN-VP-006-GOVERNANCE-IN-VISUAL-SURFACES.md` | How policies, roles, obligations, and provenance are rendered and enforced in non-textual editors. | TBD |

> **Note:** The foundational design for the spec processor workflow lives in [`../DESIGN-SPEC-PROCESSOR.md`](../DESIGN-SPEC-PROCESSOR.md) rather than in this directory, since it is a language-tooling design rather than a visual-interface design.

## Relationship to Other Specifications

- [SPEC-039](../spec/SPEC-039-PARSER-TOOLING-INFRASTRUCTURE.md) — spans and comment trivia are prerequisites for any round-trip visual editor.
- [SPEC-042](../spec/SPEC-042-ASH-SOURCE-FORMATTER.md) — the formatter is the text-export path from any graph or notebook state.
- [SPEC-021](../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) — defines what live-state projections must be faithful to.
- [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) — the execution model that visual runtime surfaces must not misrepresent.

## Research Sources

- **Smalltalk-80 / Pharo / Glamorous Toolkit** — live object inspection, browser/inspector duality, image-based persistence.
- **Node-RED / n8n** — dataflow topology, message-based node graphs, runtime deployment.
- **SCADA / HMI (Ignition Perspective, Wonderware, ladder logic)** — supervisory control, tag historization, alarm management, P&ID visual languages.
- **Jupyter / Observable** — notebook cell semantics, reactive re-evaluation, kernel architecture.
- **Scratch / Blockly / OmniBlocks** — shape-based port typing, block palette constraints, accessibility-oriented visual syntax.

## Contributing

Add new documents with the `DESIGN-VP-NNN` prefix. Keep each document focused on a single concern. Cross-reference liberally.
